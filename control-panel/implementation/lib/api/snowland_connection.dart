import 'package:snowland_control_panel/api/isolate/main_isolate_api.dart';
import 'package:snowland_control_panel/api/control_panel_api_ffi.dart' as snowland_ffi;

class DisconnectedException implements Exception {
  DisconnectedException();
}

class SnowlandConnection {
  final int _instance;
  final MainIsolateAPI _api;
  final Stream<snowland_ffi.SnowlandAPIEvent> _events;

  const SnowlandConnection(this._instance, this._api, this._events);
}
