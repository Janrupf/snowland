import 'package:flutter/material.dart';
import 'package:snowland_control_panel/com/dart_to_native.dart';
import 'package:snowland_control_panel/com/native_to_dart.dart';
import 'package:snowland_control_panel/components/module_list.dart';
import 'package:snowland_control_panel/data/configuration.dart';
import 'package:snowland_control_panel/logger.dart';

const logger = Logger("connected_view");

class ConnectedView extends StatefulWidget {
  const ConnectedView({Key? key}) : super(key: key);

  @override
  State<ConnectedView> createState() => _ConnectedViewState();
}

class _ConnectedViewState extends State<ConnectedView> {
  @override
  Widget build(BuildContext context) => Container(
        color: Theme.of(context).backgroundColor,
        child: NativeCallReceiver(
            methodName: "update_configuration",
            builder: (context, data) {
              if (data == null) {
                logger.debug("Asking daemon for configuration...");
                DartToNativeCommunicator.instance.queryConfiguration();
                return _buildWaitingForConfiguration();
              } else {
                return _buildWithConfiguration(
                  context,
                  Configuration.fromData(data),
                );
              }
            }),
      );

  Widget _buildWaitingForConfiguration() =>
      const Center(child: CircularProgressIndicator());

  Widget _buildWithConfiguration(
    BuildContext context,
    Configuration configuration,
  ) {
    logger.debug("Received configuration from daemon: $configuration");

    return Row(
      children: [
        _buildSidebar(
          context,
          configuration,
        ),
      ],
    );
  }

  Widget _buildSidebar(BuildContext context, Configuration configuration) =>
      Container(
        constraints: const BoxConstraints(maxWidth: 200),
        child: Material(
            child: ModuleList(
          configuration: configuration,
          onSelected: _onModuleSelected,
          onReorder: _onModuleReorder,
        )),
      );

  void _onModuleSelected(InstalledModule module) {}

  void _onModuleReorder(int oldIndex, int newIndex) {
    logger.debug("Moving module from position $oldIndex to $newIndex");
    DartToNativeCommunicator.instance.reorderModules(oldIndex, newIndex);
  }
}
