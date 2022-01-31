import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/parts/alignment_property_editor.dart';
import 'package:snowland_control_panel/components/parts/card/property_card.dart';
import 'package:snowland_control_panel/components/parts/color_property_editor.dart';
import 'package:snowland_control_panel/components/parts/display_property_editor.dart';
import 'package:snowland_control_panel/components/parts/number_property_editor.dart';
import 'package:snowland_control_panel/components/parts/paint_editor.dart';
import 'package:snowland_control_panel/data/property.dart';

class StandardPropertyCards {
  const StandardPropertyCards._();

  static PropertyCard color([ConfigurationPropertyList<double>? property]) {
    property ??= ConfigurationPropertyList(["paint", "color"]);

    return PropertyCard(
      title: const Text("Color"),
      child: ColorPropertyEditor(
        property: property,
      ),
    );
  }

  static PropertyCard paint([
    ConfigurationPropertyGroup baseGroup = const ConfigurationPropertyGroup(
      ["paint"],
    ),
  ]) =>
      PropertyCard(
        title: const Text("Paint settings:"),
        child: PaintEditor(
          group: baseGroup,
        ),
      );

  static PropertyCard position([
    ConfigurationPropertyGroup baseGroup = const ConfigurationPropertyGroup(
      ["position"],
    ),
  ]) =>
      PropertyCard(
        minWidth: 300,
        title: const Text("Position:"),
        child: Column(
          children: [
            ListTile(
              title: const Text("Display"),
              trailing: DisplayPropertyEditor(
                group: baseGroup.group(["display"]),
              ),
            ),
            ListTile(
              title: const Text("Horizontal alignment"),
              trailing: AlignmentPropertyEditor(
                property: baseGroup.property(["horizontal"]),
                type: AlignmentPropertyType.horizontal,
              ),
            ),
            ListTile(
              title: const Text("Vertical alignment"),
              trailing: AlignmentPropertyEditor(
                property: baseGroup.property(["vertical"]),
                type: AlignmentPropertyType.vertical,
              ),
            ),
            ListTile(
              title: const Text("X"),
              trailing: SizedBox(
                width: 200,
                child: NumberPropertyEditor<int>(
                  property: baseGroup.property(["x_offset"]),
                  signed: true,
                  draggable: true,
                ),
              ),
            ),
            ListTile(
              title: const Text("Y"),
              trailing: SizedBox(
                width: 200,
                child: NumberPropertyEditor<int>(
                  property: baseGroup.property(["y_offset"]),
                  signed: true,
                  draggable: true,
                ),
              ),
            ),
          ],
        ),
      );
}
