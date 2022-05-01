import 'dart:async';
import 'dart:ffi' as ffi;
import 'dart:isolate';
import 'package:snowland_control_panel/api/control_panel_api_ffi.dart'
    as snowland_ffi;
import 'package:snowland_control_panel/logger.dart';

const _logger = Logger("main_handler");

/// API available on the main isolate
class MainIsolateAPI {
  final snowland_ffi.ControlPanelAPIFFI _ffi;
  final ffi.Pointer<snowland_ffi.SnowlandMessageSender>? _sender;
  ffi.Pointer<snowland_ffi.SnowlandAPI>? _api;

  final ReceivePort _dartMessageReceiver;

  Completer<List<int>>? _aliveCompleter;
  final Completer<void> _shutdownCompleter;

  MainIsolateAPI(this._ffi, this._sender, this._api)
      : _dartMessageReceiver = ReceivePort("ffi-dart-message-receiver"),
        _shutdownCompleter = Completer() {
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

    return _shutdownCompleter.future;
  }

  void _handleControlMessage(dynamic controlData) {
    final data = controlData as Map<String, dynamic>;

    final connection = data["connection"] as int;
    final event = data["event"] as snowland_ffi.SnowlandAPIEvent;

    if (connection == 0) {
      _handleControlEvent(event);
    }
  }

  void _handleControlEvent(snowland_ffi.SnowlandAPIEvent event) {
    if (event is snowland_ffi.SnowlandAPIEventShutdown) {
      if (!_shutdownCompleter.isCompleted) {
        _shutdownCompleter.complete();
      }
    } else if (event is snowland_ffi.SnowlandAPIEventAliveInstances) {
      if (_aliveCompleter == null) {
        _logger.warn("Received alive instances event without requesting it!");
        return;
      }

      _aliveCompleter!.complete(event.alive);
      _aliveCompleter = null;
    } else {
      _logger.warn("Unknown control event $event");
    }
  }
}
