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
  final ffi.Pointer<snowland_ffi.SnowlandMessageSender> _sender;
  ffi.Pointer<snowland_ffi.SnowlandAPI>? _api;

  final ReceivePort _dartMessageReceiver;

  Completer<List<int>>? _aliveCompleter;

  MainIsolateAPI(this._ffi, this._sender, this._api)
      : _dartMessageReceiver = ReceivePort("ffi-dart-message-receiver") {
    _dartMessageReceiver.forEach(_handleControlMessage);
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
    _ffi.listAlive(_sender);

    return _aliveCompleter!.future;
  }

  void _handleControlMessage(dynamic controlData) {
    final map = controlData as Map<String, dynamic>;

    final messageType = map["messageType"];
    final data = map["data"];

    switch (messageType) {
      case "aliveConnections":
        {
          if (_aliveCompleter == null) {
            _logger.warn(
                "Received aliveConnections message without requesting it!");
            return;
          }

          _aliveCompleter!.complete(data);
          _aliveCompleter = null;
          break;
        }
    }
  }
}
