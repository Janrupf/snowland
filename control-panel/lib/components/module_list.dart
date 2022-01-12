import 'package:flutter/material.dart';
import 'package:snowland_control_panel/data/configuration.dart';

typedef ModuleSelectedCallback = void Function(InstalledModule mod);
typedef ModuleOrderCallback = void Function(int oldIndex, int newIndex);

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

  @override
  Widget build(BuildContext context) => Scrollbar(
      isAlwaysShown: true,
      controller: _scrollController,
      child: ReorderableListView(
          padding: const EdgeInsets.all(10),
          children: _getModules(),
          scrollController: _scrollController,
          onReorder: _handleReorder));

  List<Widget> _getModules() =>
      List.generate(widget.configuration.modules.length, (index) {
        final mod = widget.configuration.modules[index];

        return ListTile(
          key: Key("mod$index"),
          title: Text(mod.type),
          onTap: () => widget.onSelected(mod),
        );
      });

  void _handleReorder(int oldIndex, int newIndex) {
    if(oldIndex < newIndex) {
      newIndex -= 1;
    }

    setState(() {
      InstalledModule mod = widget.configuration.modules.removeAt(oldIndex);
      widget.configuration.modules.insert(newIndex, mod);
    });

    widget.onReorder(oldIndex, newIndex);
  }
}
