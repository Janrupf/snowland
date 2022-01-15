import 'package:flutter/material.dart';
import 'package:snowland_control_panel/data/configuration.dart';

typedef _ModuleEditorFactory = Widget Function(InstalledModule module);

abstract class ModuleEditor {
  ModuleEditor._();

  static const Map<String, _ModuleEditorFactory> factories = {};

  static Widget createEditor(InstalledModule module) {
    final factory = factories[module.type];

    if (factory == null) {
      return _NoEditorAvailable();
    }

    return factory(module);
  }
}

class _NoEditorAvailable extends StatelessWidget {
  @override
  Widget build(BuildContext context) => const Expanded(
        child: Center(
          child: Text(
            "Currently there is no editor available for this module",
            style: TextStyle(fontSize: 20),
            textAlign: TextAlign.center,
          ),
        ),
      );
}
