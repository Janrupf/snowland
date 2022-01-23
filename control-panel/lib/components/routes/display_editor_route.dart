import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/custom/display_selection.dart';
import 'package:snowland_control_panel/components/display_provider.dart';
import 'package:snowland_control_panel/data/display.dart';

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
        child: DisplayProvider(
          loadingBuilder: _buildLoading,
          availableBuilder: _buildAvailable,
        ),
      );

  Widget _buildLoading(BuildContext context) => Expanded(
        child: Column(
          children: [
            Center(
              child: ElevatedButton(
                child: const Text("Back"),
                onPressed: () => Navigator.of(context).pop(),
              ),
            ),
            const Center(
              // TODO: A progress indicator seems to block the event loop sometimes
              child: Text("Waiting for daemon"),
            ),
          ],
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
  Widget build(BuildContext context) {
    final displays = DisplayProvider.of(context).displays;

    return Padding(
      padding: const EdgeInsets.all(40.0),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Center(
            child: ElevatedButton(
              child: const Text("Back"),
              onPressed: () => Navigator.of(context).pop(),
            ),
          ),
          const SizedBox(height: 20),
          DropdownButton<DisplaySelection>(
            items: _collectAvailableSelections(displays),
            value: _currentSelection,
            onChanged: _changeSelection,
          ),
          const SizedBox(height: 20),
          Flexible(
            fit: FlexFit.loose,
            child: DisplaySelector(
                selection: _currentSelection,
                displays: displays,
                onChanged: _changeSelection),
          ),
        ],
      ),
    );
  }

  void _changeSelection(DisplaySelection? newSelection) {
    assert(newSelection != null, "The display selection should never be null");

    if(_currentSelection != newSelection) {
      setState(() {
        _currentSelection = newSelection!;
      });

      widget.onChanged(_currentSelection);
    }
  }

  List<DropdownMenuItem<DisplaySelection>> _collectAvailableSelections(
    List<Display> displays,
  ) =>
      [
        _itemFor(DisplaySelection.primary()),
        _itemFor(DisplaySelection.none()),
        ...displays.map(DisplaySelection.fromDisplay).map(_itemFor),
      ];

  DropdownMenuItem<DisplaySelection> _itemFor(DisplaySelection selection) =>
      DropdownMenuItem(
        value: selection,
        child: Text(selection.displayName()),
      );
}
