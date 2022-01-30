import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/parts/alignment_property_editor.dart';
import 'package:snowland_control_panel/components/parts/color_property_editor.dart';
import 'package:snowland_control_panel/components/parts/display_property_editor.dart';
import 'package:snowland_control_panel/components/parts/number_property_editor.dart';
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
                  minWidth: 300,
                  title: const Text("Position:"),
                  child: Column(
                    children: [
                      const ListTile(
                        title: Text("Display"),
                        trailing: DisplayPropertyEditor(
                          group: ConfigurationPropertyGroup(
                              ["position", "display"]),
                        ),
                      ),
                      const ListTile(
                        title: Text("Horizontal alignment"),
                        trailing: AlignmentPropertyEditor(
                          property:
                              ConfigurationProperty(["position", "horizontal"]),
                          type: AlignmentPropertyType.horizontal,
                        ),
                      ),
                      const ListTile(
                        title: Text("Vertical alignment"),
                        trailing: AlignmentPropertyEditor(
                          property:
                              ConfigurationProperty(["position", "vertical"]),
                          type: AlignmentPropertyType.vertical,
                        ),
                      ),
                      ListTile(
                        title: const Text("X"),
                        trailing: SizedBox(
                          width: 200,
                          child: NumberPropertyEditor<int>(
                            property: const ConfigurationProperty(
                              ["position", "x_offset"],
                            ),
                            draggable: true,
                          ),
                        ),
                      ),
                      ListTile(
                        title: const Text("Y"),
                        trailing: SizedBox(
                          width: 200,
                          child: NumberPropertyEditor<int>(
                            property: const ConfigurationProperty(
                                ["position", "y_offset"]
                            ),
                            draggable: true,
                          ),
                        ),
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
