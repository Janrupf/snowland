import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/parts/card/property_card.dart';
import 'package:snowland_control_panel/components/parts/card/standard_property_cards.dart';
import 'package:snowland_control_panel/components/parts/date_time_property_editor.dart';
import 'package:snowland_control_panel/data/property.dart';

class CountdownModuleEditor extends StatelessWidget {
  const CountdownModuleEditor({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => Column(
        children: [
          const PropertyCard(
            title: Text("Target"),
            child: DateTimePropertyEditor(
              property: ConfigurationProperty(
                ["target"],
              ),
            ),
          ),
          StandardPropertyCards.color(),
          StandardPropertyCards.paint(),
          StandardPropertyCards.position(),
        ],
      );
}
