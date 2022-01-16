import 'package:flutter/material.dart';
import 'package:snowland_control_panel/data/configuration.dart';
import 'package:snowland_control_panel/logger.dart';

typedef ModuleSelectedCallback = void Function(InstalledModule mod);
typedef ModuleOrderCallback = void Function(int oldIndex, int newIndex);

const _logger = Logger("module_list");

class ModuleList extends StatefulWidget {
  final ModuleSelectedCallback onSelected;
  final ModuleOrderCallback onReorder;
  final Configuration configuration;

  const ModuleList({
    Key? key,
    required this.onSelected,
    required this.onReorder,
    required this.configuration,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => _ModuleListState();
}

class _ModuleListState extends State<ModuleList> {
  final ScrollController _scrollController = ScrollController();

  int? _activeWidget;

  @override
  Widget build(BuildContext context) => Scrollbar(
      isAlwaysShown: true,
      controller: _scrollController,
      child: ReorderableListView(
        padding: const EdgeInsets.all(10),
        children: _getModules(),
        scrollController: _scrollController,
        onReorder: _handleReorder,
      ));

  List<Widget> _getModules() =>
      List.generate(widget.configuration.modules.length, (index) {
        final mod = widget.configuration.modules[index];

        final theme = ListTileTheme.of(context);
        final color =
            _activeWidget == index ? theme.selectedTileColor : theme.tileColor;
        final textColor =
            _activeWidget == index ? theme.selectedColor : theme.textColor;

        final padding = theme.contentPadding ?? const EdgeInsets.all(10);

        final Decoration? decoration;
        if (theme.shape != null) {
          decoration = ShapeDecoration(shape: theme.shape!, color: color);
        } else {
          decoration = BoxDecoration(color: color);
        }

        // This is only half of what a ListTile should really be, but
        // enough for our use case.
        //
        // ListTile's have problems with reorderable lists leading to
        // unreasonable animations and the background color sticking
        // instead of moving with the widget.
        return InkWell(
          customBorder: theme.shape,
          onTap: () {
            _activeWidget = index;

            widget.onSelected(mod);
          },
          key: ValueKey(mod),
          child: Container(
            decoration: decoration,
            alignment: Alignment.centerLeft,
            child: Padding(
              padding: padding,
              child: Text(
                mod.type,
                style: TextStyle(
                  color: textColor,
                  fontSize: 15,
                ),
              ),
            ),
            height: 40,
          ),
        );
      });

  void _handleReorder(int oldIndex, int newIndex) {
    if (oldIndex < newIndex) {
      newIndex -= 1;
    }

    setState(() {
      if (_activeWidget == oldIndex) {
        _activeWidget = newIndex;
      } else if (_activeWidget != null) {
        if (oldIndex < _activeWidget! && newIndex >= _activeWidget!) {
          _activeWidget = _activeWidget! - 1;
        } else if (oldIndex > _activeWidget! && newIndex <= _activeWidget!) {
          _activeWidget = _activeWidget! + 1;
        }
      }

      InstalledModule mod = widget.configuration.modules.removeAt(oldIndex);
      widget.configuration.modules.insert(newIndex, mod);
    });

    widget.onReorder(oldIndex, newIndex);
  }
}
