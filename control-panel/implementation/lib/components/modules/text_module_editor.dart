import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/parts/card/property_card.dart';
import 'package:snowland_control_panel/components/parts/card/standard_property_cards.dart';
import 'package:snowland_control_panel/components/parts/single_line_editor.dart';
import 'package:snowland_control_panel/data/property.dart';

class TextModuleEditor extends StatelessWidget {
  const TextModuleEditor({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => Column(
        children: [
          const PropertyCard(
            title: Text("Module Text:"),
            subtitle: Text("Snowland will display this on screen"),
            child: SingleLinePropertyEditor(
              property: ConfigurationProperty(["value"]),
            ),
          ),
          StandardPropertyCards.color(),
          StandardPropertyCards.paint(),
          StandardPropertyCards.position()
        ],
      );
}
