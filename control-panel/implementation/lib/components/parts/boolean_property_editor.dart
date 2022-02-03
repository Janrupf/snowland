import 'package:flutter/material.dart';
import 'package:snowland_control_panel/data/property.dart';

class BooleanPropertyEditor extends StatefulWidget {
  final ConfigurationProperty<bool> property;

  const BooleanPropertyEditor({
    Key? key,
    required this.property,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => _BooleanPropertyEditorState();
}

class _BooleanPropertyEditorState extends State<BooleanPropertyEditor> {
  late bool _value;

  @override
  void initState() {
    super.initState();
    _value = widget.property.obtain(context);
  }

  @override
  Widget build(BuildContext context) => Switch(
        value: _value,
        onChanged: (newValue) => setState(() {
          _value = newValue;
          widget.property.set(context, newValue);
        }),
      );
}
