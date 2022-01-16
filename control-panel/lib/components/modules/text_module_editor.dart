import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/parts/single_line_editor.dart';
import 'package:snowland_control_panel/data/property.dart';

class TextModuleEditor extends StatelessWidget {
  const TextModuleEditor({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => const Material(
        child: SingleLineEditor(property: ConfigurationProperty(["value"])),
      );
}
