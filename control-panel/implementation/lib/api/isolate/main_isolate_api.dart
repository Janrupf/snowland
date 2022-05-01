import 'dart:async';
import 'dart:collection';
import 'dart:ffi' as ffi;
import 'dart:isolate';
import 'package:snowland_control_panel/api/control_panel_api_ffi.dart'
    as snowland_ffi;
import 'package:snowland_control_panel/api/snowland_connection.dart';
import 'package:snowland_control_panel/logger.dart';

const _logger = Logger("main_handler");

/// API available on the main isolate
///
/// This class is mainly concerned with translation between the native API and
/// Dart. All established and pending API connections are managed here and
/// events are routed to their respective Dart instances.
class MainIsolateAPI {
  final snowland_ffi.ControlPanelAPIFFI _ffi;
  ffi.Pointer<snowland_ffi.SnowlandMessageSender>? _sender;
  ffi.Pointer<snowland_ffi.SnowlandAPI>? _api;

  final ReceivePort _dartMessageReceiver;

  Completer<List<int>>? _aliveCompleter;
  final Completer<void> _shutdownCompleter;
  final Map<int, Completer<Stream<snowland_ffi.SnowlandAPIEvent>>>
      _pendingConnections;
  final Map<int, StreamController<snowland_ffi.SnowlandAPIEvent>>
      _establishedConnections;

  MainIsolateAPI(this._ffi, this._sender, this._api)
      : _dartMessageReceiver = ReceivePort("ffi-dart-message-receiver"),
        _shutdownCompleter = Completer(),
        _pendingConnections = HashMap(),
        _establishedConnections = HashMap() {
    _dartMessageReceiver.forEach(_handleControlMessage);
  }

  ffi.Pointer<snowland_ffi.SnowlandMessageSender> _ensureSender() {
    if (_sender == null) {
      throw StateError("API has been shut down already");
    }

    return _sender!;
  }

  /// Exports the internal data so it can be re-imported by the
  /// handler [Isolate]
  Map<String, dynamic> exportForHandler() {
    if (_api == null) {
      throw StateError("Exported already");
    }

    final api = _api!;
    _api = null;

    return {
      "apiPtr": api.address,
      "messageSender": _dartMessageReceiver.sendPort
    };
  }

  Future<List<int>> listAlive() {
    if (_aliveCompleter != null) {
      return _aliveCompleter!.future;
    }

    _aliveCompleter = Completer();
    _ffi.listAlive(_ensureSender());

    return _aliveCompleter!.future;
  }

  Future<void> shutdown() {
    if (_sender != null) {
      _ffi.shutdown(_sender!);
    }

    _sender = null;

    return _shutdownCompleter.future;
  }

  Future<Stream<snowland_ffi.SnowlandAPIEvent>> connect(int instance) {
    final sender = _ensureSender();

    final existing = _establishedConnections[instance];
    if (existing != null) {
      return Future.value(existing.stream);
    }

    final connectionCompleter =
        Completer<Stream<snowland_ffi.SnowlandAPIEvent>>();
    _pendingConnections[instance] = connectionCompleter;

    _ffi.connect(sender, instance);

    return connectionCompleter.future;
  }

  void _handleControlMessage(dynamic controlData) {
    // This is called for every event the native API generated
    final data = controlData as Map<String, dynamic>;

    final connection = data["connection"] as int;
    final event = data["event"] as snowland_ffi.SnowlandAPIEvent;

    if (connection == 0) {
      // Connection 0 are always control events
      _handleControlEvent(event);
    } else {
      // Route the event to its respective connection,
      // intercepting state changes
      _handleConnectionEvent(connection, event);
    }
  }

  void _handleControlEvent(snowland_ffi.SnowlandAPIEvent event) {
    if (event is snowland_ffi.SnowlandAPIEventShutdown) {
      // This check really shouldn't be necessary, but better safe than sorry
      if (!_shutdownCompleter.isCompleted) {
        _shutdownCompleter.complete();
      }
    } else if (event is snowland_ffi.SnowlandAPIEventAliveInstances) {
      if (_aliveCompleter == null) {
        // How did that happen? Hot reload?
        _logger.warn("Received alive instances event without requesting it!");
        return;
      }

      _aliveCompleter!.complete(event.alive);
      _aliveCompleter = null;
    } else {
      // Possibly incompatible native library version? This should really not
      // be triggered
      _logger.error("Unknown control event $event");
    }
  }

  void _handleConnectionEvent(
    int instance,
    snowland_ffi.SnowlandAPIEvent event,
  ) {
    // Connection state changes need to be intercepted, the connection
    // can't entirely manage itself
    if (event is snowland_ffi.SnowlandAPIEventConnectionState) {
      switch (event) {
        case snowland_ffi.SnowlandAPIEventConnectionState.connected:
          {
            final pending = _pendingConnections.remove(instance);
            if (pending == null) {
              // ??? Hot reload?
              _logger.warn(
                "Received connect event for instance $instance without"
                " requesting a connection!",
              );
              return;
            }

            // Prepare the event stream
            final controller =
                StreamController<snowland_ffi.SnowlandAPIEvent>();
            _establishedConnections[instance] = controller;

            // The connection finished, we can now complete the pending state
            pending.complete(controller.stream);

            break;
          }
        case snowland_ffi.SnowlandAPIEventConnectionState.disconnected:
          {
            final pending = _pendingConnections.remove(instance);
            if (pending != null) {
              // Disconnected can be received without ever being connected in
              // case the connection fails
              pending.completeError(DisconnectedException());
              return;
            }

            final established = _establishedConnections.remove(instance);
            if (established == null) {
              // ??? Hot reload?
              _logger.warn(
                "Received disconnected event for instance $instance without"
                " ever requesting a connection!",
              );
              return;
            }

            // Connection is gone, close down the stream and release resources
            established.addError(DisconnectedException());
            established.close();

            break;
          }
      }
    } else {
      final connection = _establishedConnections[instance];
      if (connection == null) {
        // ??? Hot reload?
        _logger.warn(
          "Received event $event for connection $instance without"
          " being connected!",
        );
        return;
      }

      // Route the event directly to the connection
      connection.add(event);
    }
  }
}
