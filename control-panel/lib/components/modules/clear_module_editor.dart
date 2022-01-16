import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/parts/color_property_editor.dart';
import 'package:snowland_control_panel/data/property.dart';

class ClearModuleEditor extends StatelessWidget {
  const ClearModuleEditor({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => Center(
    child: Container(
      decoration: BoxDecoration(
          color: Theme.of(context).canvasColor,
          borderRadius: const BorderRadius.all(Radius.circular(20)),
          boxShadow: [
            BoxShadow(
                color: Theme.of(context).canvasColor.withAlpha(200),
                blurRadius: 7,
                spreadRadius: 3,
                offset: const Offset(0, 3)),
          ]),
      child: Material(
        type: MaterialType.transparency,
        child: IntrinsicWidth(
          child: IntrinsicHeight(
            child: ColorPropertyEditor(
              property: ConfigurationPropertyList(["color"]),
            ),
          ),
        ),
      ),
    ),
  );
}
