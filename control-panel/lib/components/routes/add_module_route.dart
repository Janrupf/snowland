import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/custom/module_selector.dart';

class AddModuleRoute extends StatelessWidget {
  const AddModuleRoute({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) =>
      Scaffold(
        appBar: AppBar(
          title: const Text("Select a widget to add"),
        ),
        body: Center(
          child: SingleChildScrollView(
            child: Wrap(
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                _selector(context, "Clear", Icons.clear),
                _selector(context, "Text", Icons.notes),
                _selector(context, "Countdown", Icons.av_timer),
                _selector(context, "Snow", Icons.ac_unit),
                _selector(context, "Image", Icons.image),
              ],
            ),
          ),
        ),
      );

  Widget _selector(BuildContext context, String title, IconData icon,) =>
      ModuleSelector(
        title: title,
        icon: icon,
        onPressed: () => Navigator.of(context).pop(title),
      );
}
