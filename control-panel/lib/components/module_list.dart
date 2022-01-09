import 'package:flutter/material.dart';

typedef ModuleSelectedCallback = void Function(int);

class ModuleList extends StatefulWidget {
  final ModuleSelectedCallback onSelected;

  const ModuleList({Key? key, required this.onSelected}) : super(key: key);

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

  List<Widget> _getModules() => List.generate(
      40,
      (index) => ListTile(
            key: Key("$index"),
            onTap: () => widget.onSelected(index),
            title: Text("Module $index"),
          ));

  void _handleReorder(int oldIndex, int newIndex) {}
}
