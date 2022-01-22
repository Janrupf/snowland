import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/routes/display_editor_route.dart';
import 'package:snowland_control_panel/data/display.dart';
import 'package:snowland_control_panel/data/property.dart';

class DisplayPropertyEditor extends StatefulWidget {
  final ConfigurationPropertyGroup group;

  const DisplayPropertyEditor({
    Key? key,
    required this.group,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => _DisplayPropertyEditorState();
}

class _DisplayPropertyEditorState extends State<DisplayPropertyEditor> {
  late DisplaySelection _currentSelection;

  @override
  void initState() {
    super.initState();
    _currentSelection = DisplaySelection.fromProperty(context, widget.group);
  }

  @override
  Widget build(BuildContext context) => Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(_currentSelection.displayName()),
          IconButton(
            onPressed: () => _onSelect(context),
            icon: const Icon(Icons.open_in_new),
          )
        ],
      );

  void _onSelect(BuildContext context) {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (context) => DisplayEditorRoute(
          onChanged: _onChanged,
        ),
      ),
    );
  }

  void _onChanged(DisplaySelection selection) {
    setState(() {
      _currentSelection = selection;
    });
  }
}
