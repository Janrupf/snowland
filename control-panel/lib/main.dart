import 'package:flutter/material.dart';
import 'package:nativeshell/nativeshell.dart';

void main() {
  runApp(const SnowlandControlPanel());
}

class SnowlandControlPanel extends StatelessWidget {
  const SnowlandControlPanel({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => Container(
    color: Colors.black,
    child: WindowWidget(onCreateState: (initData) {
      WindowState? state;
      state ??= SnowlandControlPanelState();
      return state;
    })
  );
}

class SnowlandControlPanelState extends WindowState {
  @override
  Widget build(BuildContext context) => const MaterialApp(
    home: WindowLayoutProbe(
      child: Center(
        child: Text("Hello, World!"),
      )
    )
  );

  @override
  WindowSizingMode get windowSizingMode => WindowSizingMode.atLeastIntrinsicSize;
}