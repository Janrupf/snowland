import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/parts/card/property_card.dart';
import 'package:snowland_control_panel/components/parts/number_range_property_editor.dart';
import 'package:snowland_control_panel/components/parts/number_slider_property_editor.dart';
import 'package:snowland_control_panel/data/property.dart';

class SnowModuleEditor extends StatelessWidget {
  const SnowModuleEditor({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => Column(
        children: [
          PropertyCard(
            title: const Text("Animation"),
            child: Column(
              children: const [
                ListTile(
                  title: Text("Fade time"),
                  subtitle: NumberSliderPropertyEditor<double>(
                    min: 0.0,
                    max: 10000.0,
                    step: 100.0,
                    property: ConfigurationProperty(["fade_time"]),
                  ),
                ),
                ListTile(
                  title: Text("Falling speed"),
                  subtitle: NumberRangePropertyEditor<double>(
                    min: 0.0,
                    max: 10.0,
                    step: 0.5,
                    minProperty: ConfigurationProperty(["falling_speed_min"]),
                    maxProperty: ConfigurationProperty(["falling_speed_max"]),
                  ),
                ),
                ListTile(
                  title: Text("Time to live"),
                  subtitle: NumberRangePropertyEditor<double>(
                    min: 1000.0,
                    max: 10000.0,
                    step: 100.0,
                    minProperty: ConfigurationProperty(["time_to_live_min"]),
                    maxProperty: ConfigurationProperty(["time_to_live_max"]),
                  ),
                ),
                ListTile(
                  title: Text("Tumbling multiplier"),
                  subtitle: NumberRangePropertyEditor<double>(
                    min: 0.0,
                    max: 2.0,
                    step: 0.05,
                    minProperty: ConfigurationProperty(["tumbling_min"]),
                    maxProperty: ConfigurationProperty(["tumbling_max"]),
                  ),
                ),
                ListTile(
                  title: Text("Pixel/flake ratio"),
                  subtitle: NumberSliderPropertyEditor<int>(
                    min: 500,
                    step: 100,
                    max: 10000,
                    property: ConfigurationProperty(["pixel_flake_ratio"]),
                  ),
                )
              ],
            ),
          )
        ],
      );
}
