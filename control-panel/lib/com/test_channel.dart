import 'package:flutter/services.dart';

const MethodChannel _testChannel = MethodChannel("test_channel");

class TestChannel {
  static Future<String?> greet(String name) {
    return _testChannel.invokeMethod<String>("greet", [name, 0.1111]);
  }
}
