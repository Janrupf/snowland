import 'dart:isolate';

import 'package:snowland_control_panel/api/control_panel_api_ffi.dart'
    as snowland_ffi;

import 'dart:ffi' as ffi;
import 'package:ffi/ffi.dart' as ffi;
import 'package:snowland_control_panel/api/isolate/handler_isolate_api.dart';
import 'package:snowland_control_panel/api/isolate/main_isolate_api.dart';
import 'package:snowland_control_panel/api/snowland_connection.dart';

/// External representation of the opaque snowland API handle
typedef SnowlandAPI = ffi.Pointer<snowland_ffi.SnowlandAPI>;

/// Exception thrown when starting a host instance failed.
class HostStartException implements Exception {
  /// The exception message
  final String message;

  const HostStartException(this.message);

  @override
  String toString() => "HostStartException: $message";
}

class ControlPanelAPI {
  static late final ControlPanelAPI _instance;

  /// Retrieves the instance of the control panel API of
  /// the current isolate
  ///
  /// This can only be used after a call to [initMainIsolate] or
  /// [initSecondaryIsolate]!
  static ControlPanelAPI get instance => _instance;

  /// Initializes the API for the main isolate
  ///
  /// This means loading the native library and initializing logging.
  static initMainIsolate() {
    final ffi = snowland_ffi.ControlPanelAPIFFI();
    ffi.initLogging();
    _instance = ControlPanelAPI._main(ffi);
  }

  /// Initializes the API for a secondary isolate
  ///
  /// This usually means that the native library is loaded already, thus
  /// logging initialization will be skipped.
  static initHandlerIsolate(Map<String, dynamic> data) {
    final ffi = snowland_ffi.ControlPanelAPIFFI();
    _instance = ControlPanelAPI._handler(ffi, data);
  }

  late final MainIsolateAPI? _mainAPI;
  late final HandlerIsolateAPI? _handlerAPI;

  final snowland_ffi.ControlPanelAPIFFI _ffi;

  ControlPanelAPI._main(this._ffi) {
    final external = _ffi.createNew(ffi.nullptr);

    _mainAPI = MainIsolateAPI(_ffi, external.sender, external.api);
    _handlerAPI = null;
  }

  ControlPanelAPI._handler(this._ffi, Map<String, dynamic> data) {
    _mainAPI = null;
    _handlerAPI = HandlerIsolateAPI.importFromData(_ffi, data);
  }

  /// Logs a message using the native rust logging api.
  void log(String component, String level, String message) {
    // Get a new local memory allocator to prevent memory leaks
    ffi.using((arena) {
      final componentPtr = component.toNativeUtf8(allocator: arena);
      final levelPtr = level.toNativeUtf8(allocator: arena);
      final messagePtr = message.toNativeUtf8(allocator: arena);
      _ffi.log(componentPtr, levelPtr, messagePtr);
    });
  }

  MainIsolateAPI _ensureMain() {
    if (_mainAPI == null) {
      throw StateError("Not on main isolate");
    } else {
      return _mainAPI!;
    }
  }

  /// Starts the handler isolate maintaining connections in the background
  void startHandlerIsolate() {
    Isolate.spawn(_handlerIsolateMain, _ensureMain().exportForHandler(),
        debugName: "apiEventDriver");
  }

  /// Lists all alive snowland instances
  Future<List<int>> listAlive() => _ensureMain().listAlive();

  /// Starts a new snowland instance and connects to it
  Future<SnowlandConnection> startNewHost() =>
      _ensureMain().startNewHost().then((host) => SnowlandConnection(
            host.instance,
            _mainAPI!,
            host.events,
          ));

  /// Stops the handler isolate and shuts down all connections
  Future<void> stopHandlerIsolate() => _ensureMain().shutdown();

  /// Initiates a connection to a specific snowland [instance]
  Future<SnowlandConnection> connect(int instance) => _ensureMain()
      .connect(instance)
      .then((events) => SnowlandConnection(instance, _mainAPI!, events));
}

void _handlerIsolateMain(Map<String, dynamic> data) {
  ControlPanelAPI.initHandlerIsolate(data);
  ControlPanelAPI.instance._handlerAPI!.enterRunLoop();
}
