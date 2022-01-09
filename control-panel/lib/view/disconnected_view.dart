import 'package:flutter/material.dart';
import 'package:snowland_control_panel/com/dart_to_native.dart';

/// View which is displayed when the snowland IPC is not connected.
class DisconnectedView extends StatelessWidget {
  const DisconnectedView({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => Container(
        color: Theme.of(context).backgroundColor,
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Text(
                "The snowland daemon is currently not running!",
                style: TextStyle(fontSize: 40),
              ),
              ElevatedButton(
                  onPressed: _onConnectPressed,
                  child: const Text("Retry connection"))
            ],
          ),
        ),
      );

  void _onConnectPressed() {
    DartToNativeCommunicator.instance.connectToIpc();
  }
}
