import 'dart:async';
import 'dart:ffi' as ffi;
import 'dart:isolate';
import 'package:snowland_control_panel/ffi/control_panel_api_ffi.dart'
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
    final map = controlData as Map<String, dynamic>;

    final messageType = map["messageType"];
    final data = map["data"];

    switch (messageType) {
      case "aliveInstances":
        {
          if (_aliveCompleter == null) {
            _logger.warn(
                "Received aliveInstances message without requesting it!");
            return;
          }

          _aliveCompleter!.complete(data);
          _aliveCompleter = null;
          break;
        }

      case "shutdown":
        {
          _shutdownCompleter.complete();
          break;
        }

      default:
        _logger.warn("Received unknown control message $messageType: $data");
        break;
    }
  }
}