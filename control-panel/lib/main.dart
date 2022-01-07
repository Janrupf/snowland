import 'package:flutter/material.dart';
import 'package:nativeshell/nativeshell.dart';
import 'package:snowland_control_panel/com/test_channel.dart';

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
        child: WindowLayoutProbe(child: TestPage()),
      ));

  @override
  WindowSizingMode get windowSizingMode =>
      WindowSizingMode.atLeastIntrinsicSize;
}

class TestPage extends StatelessWidget {
  const TestPage({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => Center(
        child: ElevatedButton(
          child: const Text("Click me!"),
          onPressed: () {
            TestChannel.test();
          },
        ),
      );
}
