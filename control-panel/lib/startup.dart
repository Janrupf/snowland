import 'package:flutter/material.dart';
import 'package:snowland_control_panel/com/dart_to_native.dart';
import 'package:snowland_control_panel/com/native_to_dart.dart';

class StartupPage extends StatefulWidget {
  const StartupPage({Key? key}) : super(key: key);

  @override
  State<StatefulWidget> createState() => _StartupPageState();
}

class _StartupPageState extends State<StartupPage> {
  bool _isConnected = false;

  @override
  Widget build(BuildContext context) => NativeCallWidget(
      methodName: "set_connected",
      handler: _setConnected,
      child: _isConnected
          ? _buildConnected(context)
          : _buildNotConnected(context));

  Widget _buildNotConnected(BuildContext context) => Center(
        child: Column(
          children: [
            const Text("Not connected"),
            ElevatedButton(
                onPressed: () {
                  DartToNativeCommunicator.instance.connectToIpc();
                },
                child: const Text("Connect"))
          ],
        ),
      );

  Widget _buildConnected(BuildContext context) => const Center(
        child: Text("Connected!"),
      );

  Future _setConnected(dynamic args) async {
    setState(() {
      _isConnected = args as bool;
    });
  }
}
