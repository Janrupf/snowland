import 'package:flutter/material.dart';
import 'package:snowland_control_panel/com/dart_to_native.dart';

/// View which is displayed when the snowland IPC is not connected.
class DisconnectedView extends StatelessWidget {
  final String? error;

  const DisconnectedView({Key? key, this.error}) : super(key: key);

  @override
  Widget build(BuildContext context) => Container(
        color: Theme.of(context).backgroundColor,
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              _buildText(context),
              ElevatedButton(
                  onPressed: _onConnectPressed,
                  child: const Text("Retry connection"))
            ],
          ),
        ),
      );

  Widget _buildText(BuildContext context) {
    if (error == null) {
      return const Text(
        "The snowland daemon is currently not running!",
        style: TextStyle(fontSize: 40),
        textAlign: TextAlign.center,
      );
    }

    return Text(
      "An error occurred while taking to the daemon: $error",
      style: TextStyle(fontSize: 40, color: Theme.of(context).errorColor),
      textAlign: TextAlign.center,
    );
  }

  void _onConnectPressed() {
    DartToNativeCommunicator.instance.connectToIpc();
  }
}
