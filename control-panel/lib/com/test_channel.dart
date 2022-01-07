import 'package:flutter/services.dart';

const MethodChannel _testChannel = MethodChannel("test_channel");

class TestChannel {
  static void test() {
    _testChannel.invokeMethod("test", ["A", "B", "C"]);
  }
}
