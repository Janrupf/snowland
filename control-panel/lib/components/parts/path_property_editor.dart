import 'package:file_selector_platform_interface/file_selector_platform_interface.dart';
import 'package:flutter/material.dart';
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
    FileSelectorPlatform.instance.openFile(
      acceptedTypeGroups: [
        XTypeGroup(
          label: "Images",
          mimeTypes: [
            "image/bmp",
            "image/gif",
            "image/x-icon",
            "image/vnd.microsoft.icon",
            "image/heic",
            "image/heif",
            "image/jpe",
            "image/jpg",
            "image/jpeg",
            "image/png",
            "image/vnd.wap.wbmp",
            "image/webp"
          ]
        ),
        XTypeGroup(
          label: "All files",
          extensions: ["*"]
        )
      ]
    ).then((selected) {
      if(selected != null && mounted) {
        setState(() {
          _controller.text = selected.path;
          widget.property.set(context, _controller.text);
        });
      }
    });
  }

  void _onChanged(String newValue) {
    widget.property.set(context, newValue);
  }
}
