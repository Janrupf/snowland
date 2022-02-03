import 'package:flutter/material.dart';
import 'package:snowland_control_panel/data/property.dart';

class SingleLinePropertyEditor extends StatefulWidget {
  final ConfigurationProperty<String> property;

  const SingleLinePropertyEditor({Key? key, required this.property})
      : super(key: key);

  @override
  State<StatefulWidget> createState() => _SingleLinePropertyEditorState();
}

class _SingleLinePropertyEditorState extends State<SingleLinePropertyEditor> {
  late final String _initialValue;
  late final TextEditingController _controller;

  @override
  void initState() {
    super.initState();
    _initialValue = widget.property.obtain(context);
    _controller = TextEditingController(text: _initialValue);
  }

  @override
  Widget build(BuildContext context) => TextField(
        controller: _controller,
        onChanged: (v) => widget.property.set(context, v),
      );
}
