import 'package:flutter/material.dart';
import 'package:nativeshell/nativeshell.dart';
import 'package:snowland_control_panel/startup.dart';

void main() {
  runApp(const SnowlandControlPanel());
}

class SnowlandControlPanel extends StatelessWidget {
  const SnowlandControlPanel({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => Container(
      color: Colors.white,
      child: WindowWidget(onCreateState: (initData) {
        WindowState? state;
        state ??= SnowlandControlPanelState();
        return state;
      }));
}

class SnowlandControlPanelState extends WindowState {
  @override
  Widget build(BuildContext context) => const MaterialApp(
          home: DefaultTextStyle(
        style: TextStyle(
          fontSize: 14,
        ),
        child: WindowLayoutProbe(child: StartupPage()),
      ));

  @override
  WindowSizingMode get windowSizingMode =>
      WindowSizingMode.atLeastIntrinsicSize;
}

