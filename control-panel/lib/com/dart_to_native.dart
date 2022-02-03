import 'package:flutter/services.dart';

const MethodChannel _dartToNativeChannel =
    MethodChannel("snowland_dart_to_native");

class FileDialogFilter {
  final String name;
  final List<String> extensions;

  const FileDialogFilter({required this.name, required this.extensions});

  Map<String, dynamic> toData() => {"name": name, "extensions": extensions};
}

class DartToNativeCommunicator {
  DartToNativeCommunicator._();

  static final DartToNativeCommunicator instance = DartToNativeCommunicator._();

  void connectToIpc() {
    _dartToNativeChannel.invokeMethod("connect_to_ipc");
  }

  void log(String component, String level, String message) {
    _dartToNativeChannel.invokeMethod("log", [component, level, message]);
  }

  void queryConfiguration() {
    _dartToNativeChannel.invokeMethod("query_configuration");
  }

  void reorderModules(int oldIndex, int newIndex) {
    _dartToNativeChannel.invokeMethod("reorder_modules", [oldIndex, newIndex]);
  }

  void updateConfiguration(int module, Map newConfiguration) {
    _dartToNativeChannel
        .invokeMethod("change_configuration", [module, newConfiguration]);
  }

  void addModule(String type) {
    _dartToNativeChannel.invokeMethod("add_module", type);
  }

  void removeModule(int module) {
    _dartToNativeChannel.invokeMethod("remove_module", module);
  }

  Future startDaemon() => _dartToNativeChannel.invokeMethod("start_daemon");

  Future<String?> openSingleFile(List<FileDialogFilter> filters) {
    return _dartToNativeChannel.invokeMethod(
      "open_single_file",
      filters.map((f) => f.toData()).toList(growable: false),
    );
  }
}
