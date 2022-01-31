import 'package:flutter/material.dart';
import 'package:snowland_control_panel/com/dart_to_native.dart';
import 'package:snowland_control_panel/com/native_to_dart.dart';
import 'package:snowland_control_panel/components/module_list.dart';
import 'package:snowland_control_panel/components/modules/module_editor.dart';
import 'package:snowland_control_panel/components/routes/add_module_route.dart';
import 'package:snowland_control_panel/data/configuration.dart';
import 'package:snowland_control_panel/data/property.dart';
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
      // TODO: The progress indicator seems to block the event loop sometimes,
      // causing the the update_configuration method never to be invoked.
      const Center(/*child: CircularProgressIndicator()*/);

  Widget _buildWithConfiguration(
    BuildContext context,
    Configuration configuration,
  ) {
    logger.debug("Received configuration from daemon: $configuration");
    return _ConnectedViewConfigurationContainer(configuration: configuration);
  }
}

class _ConnectedViewConfigurationContainer extends StatefulWidget {
  final Configuration configuration;

  const _ConnectedViewConfigurationContainer({
    Key? key,
    required this.configuration,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() =>
      _ConnectedViewConfigurationContainerState();
}

class _ConnectedViewConfigurationContainerState
    extends State<_ConnectedViewConfigurationContainer> {
  InstalledModule? _selectedModule;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        _buildSidebar(context),
        _buildConfigurationArea(context),
      ],
    );
  }

  Widget _buildSidebar(BuildContext context) => Container(
        constraints: const BoxConstraints(maxWidth: 200),
        child: Material(
            child: Column(
          mainAxisSize: MainAxisSize.max,
          children: [
            Expanded(
              child: ModuleList(
                configuration: widget.configuration,
                onSelected: _onModuleSelected,
                onReorder: _onModuleReorder,
              ),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 20.0),
              child: ElevatedButton(
                onPressed: () => _onAddModuleClicked(context),
                child: const Text("Add new module"),
              ),
            ),
          ],
        )),
      );

  Widget _buildConfigurationArea(BuildContext context) {
    if (_selectedModule == null) {
      return const Expanded(
        child: Center(
          child: Text(
            "Select a module on the left to configure",
            style: TextStyle(fontSize: 20),
            textAlign: TextAlign.center,
          ),
        ),
      );
    }

    return Expanded(
      child: LayoutBuilder(
        builder: (context, viewportConstraints) => SingleChildScrollView(
          child: ConstrainedBox(
            constraints: BoxConstraints(
              minHeight: viewportConstraints.maxHeight,
            ),
            child: ConfigurationProvider(
              configuration: _selectedModule!.configuration,
              onChange: _onConfigurationChanged,
              child: ModuleEditor.createEditor(_selectedModule!.type),
            ),
          ),
        ),
      ),
    );
  }

  void _onModuleSelected(InstalledModule module) {
    setState(() {
      _selectedModule = module;
    });
  }

  void _onModuleReorder(int oldIndex, int newIndex) {
    logger.debug("Moving module from position $oldIndex to $newIndex");
    DartToNativeCommunicator.instance.reorderModules(oldIndex, newIndex);
  }

  void _onConfigurationChanged() {
    if (_selectedModule == null) {
      logger
          .error("Tried to update configuration while no module was selected!");
      return;
    }

    final idx = widget.configuration.modules.indexOf(_selectedModule!);
    if (idx < 0) {
      logger.error(
          "Module to update configuration for not found in installed modules!");
      return;
    }

    logger.trace("Configuration changed for module ${_selectedModule!.type}");
    DartToNativeCommunicator.instance
        .updateConfiguration(idx, _selectedModule!.configuration);
  }

  void _onAddModuleClicked(BuildContext context) {
    Navigator.of(context).push(MaterialPageRoute<String>(
      builder: (context) => const AddModuleRoute(),
    )).then((moduleToAdd) {
      if(moduleToAdd != null) {
        DartToNativeCommunicator.instance.addModule(moduleToAdd);
      }
    });
  }
}
