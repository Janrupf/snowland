import 'dart:ffi' as ffi;
import 'dart:isolate';
import 'package:snowland_control_panel/ffi/control_panel_api_ffi.dart'
    as snowland_ffi;
import 'package:snowland_control_panel/logger.dart';

const _logger = Logger("api_handler");

class HandlerIsolateAPI {
  final snowland_ffi.ControlPanelAPIFFI _ffi;
  late final ffi.Pointer<snowland_ffi.SnowlandAPI> _api;
  late final SendPort _dartMessageSender;

  HandlerIsolateAPI.importFromData(this._ffi, Map<String, dynamic> data) {
    _api = ffi.Pointer.fromAddress(data["apiPtr"]);
    _dartMessageSender = data["messageSender"];
  }

  /// Enters the run loop on the current [Isolate]
  ///
  /// This may only be called on secondary control panel API instances created
  /// using [initSecondaryIsolate].
  void enterRunLoop() {
    _logger.debug("Starting handler event loop");

    // Schedule a polling iteration to be run in the next event loop iteration
    Future(_runLoopPoll);
  }

  void _runLoopPoll() {
    final events = _ffi.poll(_api);
    if (events == ffi.nullptr) {
      // Self schedule polling to be run
      Future(_runLoopPoll);
      return;
    }

    bool keepRunning = true;

    try {
      final count = _ffi.eventCount(events);

      for (int i = 0; i < count; i++) {
        final connection = _ffi.getEventConnectionId(events, i);
        final data = _ffi.getEventData(events, i);

        final event = data.ref.decode();
        keepRunning &= _handleEvent(connection, event);
      }
    } finally {
      _ffi.freeEvents(events);
    }

    if (keepRunning) {
      // Self schedule polling to be run
      Future(_runLoopPoll);
      return;
    }

    _logger.trace("Ending handler isolate");
    _ffi.free(_api);
  }

  bool _handleEvent(int connection, snowland_ffi.SnowlandAPIEvent event) {
    _logger.trace("Received event for connection $connection: $event");

    if (connection == 0) {
      return _handleControlEvent(event);
    }

    return true;
  }

  bool _handleControlEvent(snowland_ffi.SnowlandAPIEvent event) {
    if (event is snowland_ffi.SnowlandAPIEventAliveInstances) {
      _dartMessageSender.send({
        "messageType": "aliveInstances",
        "data": event.alive,
      });
    } else if (event is snowland_ffi.SnowlandAPIEventShutdown) {
      _dartMessageSender.send({
        "messageType": "shutdown",
        "data": null,
      });

      return false;
    }

    return true;
  }
}
