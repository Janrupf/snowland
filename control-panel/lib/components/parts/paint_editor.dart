import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/parts/boolean_property_editor.dart';
import 'package:snowland_control_panel/components/parts/stroke_property_editor.dart';
import 'package:snowland_control_panel/data/property.dart';

class PaintEditor extends StatelessWidget {
  final ConfigurationPropertyGroup group;

  const PaintEditor({Key? key, required this.group}) : super(key: key);

  @override
  Widget build(BuildContext context) => Column(
        children: [
          ListTile(
            title: const Text("Anti alias"),
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                BooleanPropertyEditor(
                  property: group.property(["anti_alias"]),
                ),
                IconButton(
                  onPressed: _antiAliasHelp,
                  icon: const Icon(Icons.help),
                )
              ],
            ),
          ),
          ListTile(
            title: const Text("Dither"),
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                BooleanPropertyEditor(
                  property: group.property(["dither"]),
                ),
                IconButton(
                  onPressed: _ditherHelp,
                  icon: const Icon(Icons.help),
                )
              ],
            ),
          ),
          const StrokePropertyEditor(
            group: ConfigurationPropertyGroup(
              ["paint", "stroke"],
            ),
          )
        ],
      );

  void _antiAliasHelp() {}

  void _ditherHelp() {}
}