import 'package:flutter/services.dart';

const MethodChannel _dartToNativeChannel =
    MethodChannel("snowland_dart_to_native");

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
}
