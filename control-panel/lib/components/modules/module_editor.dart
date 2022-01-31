import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/modules/clear_module_editor.dart';
import 'package:snowland_control_panel/components/modules/countdown_module_editor.dart';
import 'package:snowland_control_panel/components/modules/text_module_editor.dart';

abstract class ModuleEditor {
  ModuleEditor._();

  static const Map<String, Widget> factories = {
    "Clear": ClearModuleEditor(),
    "Text": TextModuleEditor(),
    "Countdown": CountdownModuleEditor(),
  };

  static Widget createEditor(String type) {
    final widget = factories[type];

    if (widget == null) {
      return _NoEditorAvailable();
    }

    return Padding(
      padding: const EdgeInsets.all(40),
      child: widget,
    );
  }
}

class _NoEditorAvailable extends StatelessWidget {
  @override
  Widget build(BuildContext context) => const Center(
        child: Text(
          "Currently there is no editor available for this module",
          style: TextStyle(fontSize: 20),
          textAlign: TextAlign.center,
        ),
      );
}
