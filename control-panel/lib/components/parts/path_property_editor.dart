import 'package:flutter/material.dart';
import 'package:snowland_control_panel/com/dart_to_native.dart';
import 'package:snowland_control_panel/data/property.dart';

class PathPropertyEditor extends StatefulWidget {
  final ConfigurationProperty<String> property;

  const PathPropertyEditor({Key? key, required this.property})
      : super(key: key);

  @override
  State<StatefulWidget> createState() => _PathPropertyEditorState();
}

class _PathPropertyEditorState extends State<PathPropertyEditor> {
  late final TextEditingController _controller;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.property.obtain(context));
  }

  @override
  Widget build(BuildContext context) => Row(
        children: [
          Expanded(
            child: TextField(
              controller: _controller,
              onChanged: _onChanged,
            ),
          ),
          ElevatedButton(onPressed: _onChoosePressed, child: const Text("...")),
        ],
      );

  void _onChoosePressed() {
    DartToNativeCommunicator.instance.openSingleFile([
      const FileDialogFilter(
        name: "Images",
        extensions: [
          "bmp",
          "gif",
          "ico",
          "heic",
          "heif",
          "jpe",
          "jpeg",
          "jpg",
          "png",
          "wbmp",
          "webp"
        ],
      ),
    ]).then((selected) {
      if (selected != null && mounted) {
        setState(() {
          _controller.text = selected;
          widget.property.set(context, _controller.text);
        });
      }
    });
  }

  void _onChanged(String newValue) {
    widget.property.set(context, newValue);
  }
}
