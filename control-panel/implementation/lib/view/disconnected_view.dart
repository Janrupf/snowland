import 'package:flutter/material.dart';

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
              Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  ElevatedButton(
                    onPressed: _onConnectPressed,
                    child: const Text("Retry connection"),
                  ),
                  ElevatedButton(
                    onPressed: () => _onStartPressed(context),
                    child: const Text("Start daemon"),
                  )
                ],
              ),
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
    // TODO: Connect
    // DartToNativeCommunicator.instance.connectToIpc();
  }

  void _onStartPressed(BuildContext context) {
    // TODO: Start daemon
    /* DartToNativeCommunicator.instance.startDaemon().then((value) {
      const snackBar = SnackBar(content: Text("Daemon started!"));
      ScaffoldMessenger.of(context).showSnackBar(snackBar);

      Future.delayed(const Duration(milliseconds: 50), _onConnectPressed);
    }, onError: (error) {
      final snackBar = SnackBar(
        content: Text("Failed to start daemon: $error"),
      );
      ScaffoldMessenger.of(context).showSnackBar(snackBar);
    }); */
  }
}
