import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/parts/color_property_editor.dart';
import 'package:snowland_control_panel/components/parts/display_property_editor.dart';
import 'package:snowland_control_panel/components/parts/paint_editor.dart';
import 'package:snowland_control_panel/components/parts/property_card.dart';
import 'package:snowland_control_panel/components/parts/single_line_editor.dart';
import 'package:snowland_control_panel/data/property.dart';

class TextModuleEditor extends StatelessWidget {
  const TextModuleEditor({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => Align(
        alignment: Alignment.center,
        child: Wrap(
          alignment: WrapAlignment.center,
          children: [
            Wrap(
              direction: Axis.vertical,
              children: const [
                PropertyCard(
                  minWidth: 500,
                  title: Text("Module Text:"),
                  subtitle: Text("Snowland will display this on screen"),
                  child: SingleLinePropertyEditor(
                    property: ConfigurationProperty(["value"]),
                  ),
                ),
                PropertyCard(
                    minWidth: 500,
                    title: Text("Paint settings:"),
                    child: PaintEditor(
                      group: ConfigurationPropertyGroup(["paint"]),
                    )),
              ],
            ),
            Wrap(
              direction: Axis.vertical,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                PropertyCard(
                  child: ColorPropertyEditor(
                    property: ConfigurationPropertyList(["paint", "color"]),
                  ),
                ),
                PropertyCard(
                  title: const Text("Position:"),
                  child: Row(
                    children: const [
                      Text("Display: "),
                      DisplayPropertyEditor(
                        group:
                            ConfigurationPropertyGroup(["position", "display"]),
                      ),
                    ],
                  ),
                )
              ],
            ),
          ],
        ),
      );
}
