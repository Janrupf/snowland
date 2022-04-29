import 'package:snowland_control_panel/ffi/control_panel_api_ffi.dart';

import 'package:ffi/ffi.dart' as ffi;

class ControlPanelAPI {
  static late final ControlPanelAPI _instance;
  static ControlPanelAPI get instance => _instance;

  static initMainIsolate() {
    final ffi = ControlPanelAPIFFI();
    ffi.initLogging();
    _instance = ControlPanelAPI._(ffi);
  }

  final ControlPanelAPIFFI _ffi;

  ControlPanelAPI._(this._ffi);

  void log(String component, String level, String message) {
    ffi.using((arena) {
      final componentPtr = component.toNativeUtf8(allocator: arena);
      final levelPtr = level.toNativeUtf8(allocator: arena);
      final messagePtr = message.toNativeUtf8(allocator: arena);
      _ffi.log(componentPtr, levelPtr, messagePtr);
    });
  }
}