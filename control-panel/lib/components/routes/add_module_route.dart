import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/custom/module_selector.dart';
import 'package:snowland_control_panel/components/modules/module_registry.dart';

class AddModuleRoute extends StatelessWidget {
  const AddModuleRoute({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => Scaffold(
        appBar: AppBar(
          title: const Text("Select a widget to add"),
        ),
        body: Center(
          child: SingleChildScrollView(
            child: Wrap(
              crossAxisAlignment: WrapCrossAlignment.center,
              children: ModuleRegistry.all()
                  .map((m) => _selector(context, m.type, m.icon))
                  .toList(growable: false),
            ),
          ),
        ),
      );

  Widget _selector(
    BuildContext context,
    String title,
    IconData icon,
  ) =>
      ModuleSelector(
        title: title,
        icon: icon,
        onPressed: () => Navigator.of(context).pop(title),
      );
}
