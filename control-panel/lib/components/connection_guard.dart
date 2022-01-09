import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:snowland_control_panel/logger.dart';

const ipcStateChannel = EventChannel("ipc_state_event");
const connectionGuardLogger = Logger("connection_guard");

/// Helper widget for displaying different widgets based on the connection state
/// of the IPC.
class IPCConnectionGuard extends StatelessWidget {
  /// Builder called when the IPC is disconnected.
  final WidgetBuilder disconnectedBuilder;

  /// Builder called when the IPC errored.
  final WidgetBuilder erroredBuilder;

  /// Builder called when the IPC is connected.
  final WidgetBuilder connectedBuilder;

  const IPCConnectionGuard(
      {Key? key,
      required this.disconnectedBuilder,
      required this.erroredBuilder,
      required this.connectedBuilder})
      : super(key: key);

  @override
  Widget build(BuildContext context) => StreamBuilder(
      stream: ipcStateChannel.receiveBroadcastStream(),
      builder: (context, snapshot) {
        if (!snapshot.hasData) {
          return disconnectedBuilder(context);
        }

        if(snapshot.data == "NotRunning") {
          return disconnectedBuilder(context);
        } else if(snapshot.data == "Running") {
          return connectedBuilder(context);
        }

        connectionGuardLogger
            .debug("Received ipc state change ${snapshot.data.runtimeType}");
        return disconnectedBuilder(context);
      });
}
