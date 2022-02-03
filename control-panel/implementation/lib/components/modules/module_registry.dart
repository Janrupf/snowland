import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/modules/clear_module_editor.dart';
import 'package:snowland_control_panel/components/modules/countdown_module_editor.dart';
import 'package:snowland_control_panel/components/modules/image_module_editor.dart';
import 'package:snowland_control_panel/components/modules/snow_module_editor.dart';
import 'package:snowland_control_panel/components/modules/text_module_editor.dart';

class _NoEditorAvailable extends StatelessWidget {
  @override
  Widget build(BuildContext context) => const Center(
        child: Text(
          "Currently there is no editor available for this module",
          style: TextStyle(fontSize: 20),
          textAlign: TextAlign.center,
        ),
      );
}

class ModuleRegistry {
  static int _nextKey = 0;

  static const Map<String, ModuleMetadata> _modules = {
    "Clear": ModuleMetadata("Clear", Icons.clear, ClearModuleEditor.new),
    "Text": ModuleMetadata("Text", Icons.notes, TextModuleEditor.new),
    "Countdown": ModuleMetadata("Countdown", Icons.av_timer, CountdownModuleEditor.new),
    "Snow": ModuleMetadata("Snow", Icons.ac_unit, SnowModuleEditor.new),
    "Image": ModuleMetadata("Image", Icons.image, ImageModuleEditor.new),
  };

  static Widget createEditor(String type) {
    final meta = metadata(type);
    if(meta == null) {
      return _NoEditorAvailable();
    }

    return Padding(
      key: ValueKey(_nextKey++),
      padding: const EdgeInsets.all(40.0),
      child: meta.factory(),
    );
  }

  static ModuleMetadata? metadata(String type) => _modules[type];

  static Iterable<ModuleMetadata> all() => _modules.values;
}

typedef ModuleEditorFactory = Widget Function();

@immutable
class ModuleMetadata {
  final String type;
  final IconData icon;
  final ModuleEditorFactory factory;

  const ModuleMetadata(this.type, this.icon, this.factory);
}
