import 'package:flex_color_picker/flex_color_picker.dart';
import 'package:flutter/material.dart';
import 'package:snowland_control_panel/data/property.dart';

class ColorPropertyEditor extends StatefulWidget {
  final bool enableOpacity;
  final ConfigurationPropertyList<double> property;

  const ColorPropertyEditor({
    Key? key,
    this.enableOpacity = false,
    required this.property,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => _ColorPropertyEditorState();
}

class _ColorPropertyEditorState extends State<ColorPropertyEditor> {
  late final Color _initialColor;

  @override
  void initState() {
    super.initState();
    _initialColor = _decodeColor(widget.property.obtain(context));
  }

  @override
  Widget build(BuildContext context) => ColorPicker(
    color: _initialColor,
    showColorCode: true,
    colorCodeHasColor: true,
    enableOpacity: widget.enableOpacity,
    opacityTrackHeight: 20,
    copyPasteBehavior: ColorPickerCopyPasteBehavior(
      copyFormat: widget.enableOpacity
          ? ColorPickerCopyFormat.hexAARRGGBB
          : ColorPickerCopyFormat.numHexRRGGBB,
    ),
    enableShadesSelection: false,
    pickersEnabled: const {
      ColorPickerType.primary: false,
      ColorPickerType.accent: false,
      ColorPickerType.wheel: true,
    },
    onColorChanged: (newColor) =>
        widget.property.set(context, _encodeColor(newColor)),
  );

  Color _decodeColor(List<double> data) {
    final r = data[0];
    final g = data[1];
    final b = data[2];
    final o = data[3];

    return Color.fromRGBO(
        (r * 255).toInt(), (g * 255).toInt(), (b * 255).toInt(), o);
  }

  List<double> _encodeColor(Color color) {
    final r = color.red.toDouble() / 255.0;
    final g = color.green.toDouble() / 255.0;
    final b = color.blue.toDouble() / 255.0;
    final o = color.opacity;

    return [r, g, b, o];
  }
}
