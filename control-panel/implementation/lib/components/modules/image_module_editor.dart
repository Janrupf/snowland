import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/parts/card/property_card.dart';
import 'package:snowland_control_panel/components/parts/card/standard_property_cards.dart';
import 'package:snowland_control_panel/components/parts/color_property_editor.dart';
import 'package:snowland_control_panel/components/parts/paint_editor.dart';
import 'package:snowland_control_panel/components/parts/path_property_editor.dart';
import 'package:snowland_control_panel/data/property.dart';

class ImageModuleEditor extends StatelessWidget {
  const ImageModuleEditor({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => Column(
        children: [
          const PropertyCard(
            title: Text("Path"),
            child: PathPropertyEditor(
              property: ConfigurationProperty(["path"]),
            ),
          ),
          StandardPropertyCards.position(),
          const PropertyCard(
            title: Text("Paint override"),
            child: _ImagePaintPropertyEditor(
              enabledProperty: ConfigurationProperty(["paint_enabled"]),
              paintGroup: ConfigurationPropertyGroup(["paint"]),
            ),
          ),
        ],
      );
}

class _ImagePaintPropertyEditor extends StatefulWidget {
  final ConfigurationProperty<bool> enabledProperty;
  final ConfigurationPropertyGroup paintGroup;

  const _ImagePaintPropertyEditor(
      {Key? key, required this.enabledProperty, required this.paintGroup})
      : super(key: key);

  @override
  State<StatefulWidget> createState() => _ImagePaintPropertyEditorState();
}

class _ImagePaintPropertyEditorState extends State<_ImagePaintPropertyEditor> {
  late bool _enabled;

  @override
  void initState() {
    super.initState();
    _enabled = widget.enabledProperty.obtain(context);
  }

  @override
  Widget build(BuildContext context) => Column(
        children: [
          ListTile(
            title: const Text("Enable paint override"),
            trailing: Switch(
              value: _enabled,
              onChanged: (newValue) => setState(() {
                _enabled = newValue;
                widget.enabledProperty.set(context, newValue);
              }),
            ),
          ),
          AnimatedSize(
            duration: const Duration(milliseconds: 400),
            curve: Curves.easeInOut,
            child: Column(
              children: [
                if (_enabled) PaintEditor(group: widget.paintGroup),
                if (_enabled)
                  ColorPropertyEditor(
                    enableOpacity: true,
                    property: widget.paintGroup.listProperty(["color"]),
                  )
              ],
            ),
          )
        ],
      );
}
