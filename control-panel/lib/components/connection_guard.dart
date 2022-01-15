import 'dart:collection';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:snowland_control_panel/logger.dart';
import 'package:snowland_control_panel/view/connected_view.dart';

const ipcStateChannel = EventChannel("ipc_state_event");
const connectionGuardLogger = Logger("connection_guard");

typedef IPCErrorWidgetBuilder = Widget Function(BuildContext context, String message);

/// Helper widget for displaying different widgets based on the connection state
/// of the IPC.
class IPCConnectionGuard extends StatelessWidget {
  /// Builder called when the IPC is disconnected.
  final WidgetBuilder disconnectedBuilder;

  /// Builder called when the IPC errored.
  final IPCErrorWidgetBuilder erroredBuilder;

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

        final data = snapshot.data;

        switch(data) {
          case "NotRunning":
            return disconnectedBuilder(context);

          case "Running":
            return connectedBuilder(context);

          default: // Error
            if(data is! LinkedHashMap) {
              connectionGuardLogger.error("Received invalid error value $data");
              return erroredBuilder(context, "Unknown error");
            } else {
              return erroredBuilder(context, data["Errored"]);
            }
        }
      });
}
