import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/parts/color_property_editor.dart';
import 'package:snowland_control_panel/data/property.dart';

class ClearModuleEditor extends StatefulWidget {
  const ClearModuleEditor({Key? key}) : super(key: key);

  @override
  State<StatefulWidget> createState() => _ClearModuleEditorState();
}

class _ClearModuleEditorState extends State<ClearModuleEditor> {
  @override
  Widget build(BuildContext context) => ColorPropertyEditor(
    property: ConfigurationPropertyList(["color"]),
  );
}
