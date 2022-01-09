import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/module_list.dart';
import 'package:snowland_control_panel/logger.dart';

const logger = Logger("connected_view");

class ConnectedView extends StatefulWidget {
  const ConnectedView({Key? key}) : super(key: key);

  @override
  State<ConnectedView> createState() => _ConnectedViewState();
}

class _ConnectedViewState extends State<ConnectedView> {
  Widget? _configurationWidget;

  @override
  Widget build(BuildContext context) => Container(
      color: Theme.of(context).backgroundColor,
      child: Row(
        children: [
          _buildSidebar(context),
          Expanded(
            child: Container(
              color: Colors.green,
            ),
          )
        ],
      ));

  Widget _buildSidebar(BuildContext context) => Container(
        constraints: const BoxConstraints(maxWidth: 200),
        child: Material(
            child: ModuleList(
          onSelected: _onWidgetSelected,
        )),
      );

  void _onWidgetSelected(int widget) {
    logger.debug("Selected widget $widget");
  }
}
