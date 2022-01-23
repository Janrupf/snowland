import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/custom/display_selection.dart';
import 'package:snowland_control_panel/components/display_provider.dart';
import 'package:snowland_control_panel/data/display.dart';

typedef DisplaySelectionCallback = void Function(DisplaySelection newSelection);

class DisplayEditorRoute extends StatelessWidget {
  final DisplaySelection currentSelection;
  final DisplaySelectionCallback onChanged;

  const DisplayEditorRoute({
    Key? key,
    required this.currentSelection,
    required this.onChanged,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) => Material(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Center(
              child: ElevatedButton(
                child: const Text("Back"),
                onPressed: () => Navigator.of(context).pop(),
              ),
            ),
            DisplayProvider(
              loadingBuilder: _buildLoading,
              availableBuilder: _buildAvailable,
            )
          ],
        ),
      );

  Widget _buildLoading(BuildContext context) => const Expanded(
        child: Center(
          // TODO: A progress indicator seems to block the event loop sometimes
          child: Text("Waiting for daemon"),
        ),
      );

  Widget _buildAvailable(BuildContext context) => _DisplayEditor(
        onChanged: onChanged,
        currentSelection: currentSelection,
      );
}

class _DisplayEditor extends StatefulWidget {
  final DisplaySelection currentSelection;
  final DisplaySelectionCallback onChanged;

  const _DisplayEditor({
    Key? key,
    required this.currentSelection,
    required this.onChanged,
  }) : super(key: key);

  @override
  State<_DisplayEditor> createState() => _DisplayEditorState();
}

class _DisplayEditorState extends State<_DisplayEditor> {
  late DisplaySelection _currentSelection;

  @override
  void initState() {
    super.initState();
    _currentSelection = widget.currentSelection;
  }

  @override
  Widget build(BuildContext context) => Expanded(
        child: Padding(
          padding: const EdgeInsets.all(40.0),
          child: DisplaySelector(
            selection: _currentSelection,
            displays: DisplayProvider.of(context).displays,
          ),
        ),
      );
}
