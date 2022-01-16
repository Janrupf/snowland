import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/parts/number_property_editor.dart';
import 'package:snowland_control_panel/data/property.dart';

class StrokePropertyEditor extends StatefulWidget {
  final ConfigurationPropertyGroup group;

  const StrokePropertyEditor({
    Key? key,
    required this.group,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => _StrokePropertyEditorState();
}

class _StrokePropertyEditorState extends State<StrokePropertyEditor> {
  late bool _enabled;

  @override
  void initState() {
    super.initState();
    final value = widget.group.self().obtain(context);
    _enabled = value != null;
  }

  @override
  Widget build(BuildContext context) => Column(
        children: [
          _buildToggle(context),
          AnimatedSize(
            duration: const Duration(milliseconds: 400),
            curve: Curves.easeInOut,
            child: Column(
              children: [if (_enabled) ..._buildControl(context)],
            ),
          ),
        ],
      );

  Widget _buildToggle(BuildContext context) => ListTile(
        title: const Text("Stroke"),
        trailing: Switch(
          value: _enabled,
          onChanged: (newValue) => setState(() {
            _enabled = newValue;

            if (_enabled) {
              widget.group.self().set(context, {"width": 0.0, "miter": 0.0});
            } else {
              widget.group.self().set(context, null);
            }
          }),
        ),
      );

  List<Widget> _buildControl(BuildContext context) => [
        ListTile(
          title: Row(
            children: [
              const Text("Width:"),
              const Spacer(),
              Expanded(
                child: NumberPropertyEditor(
                  property: widget.group.property(["width"]),
                ),
              ),
            ],
          ),
        ),
        ListTile(
          title: Row(
            children: [
              const Text("Miter:"),
              const Spacer(),
              Expanded(
                child: NumberPropertyEditor(
                  property: widget.group.property(["miter"]),
                ),
              )
            ],
          ),
        ),
      ];
}
