import 'dart:async';

import 'package:flutter/material.dart';
import 'package:snowland_control_panel/ffi/control_panel_api.dart';
import 'package:snowland_control_panel/logger.dart';
import 'package:snowland_control_panel/theme/dark.dart';
import 'package:snowland_control_panel/view/main_view_wrapper.dart';

/// Entry point of the application, runs **after** rust has started
/// executing!
///
/// The main function here takes care of setting up a zone, in which
/// we catch print messages and delegate them to a rust logger.
void main() => runZoned(() {
      ControlPanelAPI.initMainIsolate();
      mainLogger.debug("Native API has been loaded!");

      runApp(const SnowlandControlPanel());
    }, zoneSpecification: _buildRootZone());

/// Global logger which catches messages printed using [print].
const printLogger = Logger("print");

/// Builds a zone specification for our application root zone.
///
/// This zone delegates print messages to the [printLogger] in order
/// to integrate them with the rust logging system. Messages logged
/// using [print] are logged as debug messages.
ZoneSpecification _buildRootZone() =>
    ZoneSpecification(print: (self, parent, caller, message) {
      final trace = StackTrace.current.toString();
      printLogger.debug("$message\n$trace");
    });

/// Main logger for everything in this file.
const mainLogger = Logger("main");

class SnowlandControlPanel extends StatelessWidget {
  const SnowlandControlPanel({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) => MaterialApp(
      theme: ThemeData.light(),
      darkTheme: DarkTheme.data(),
      themeMode: ThemeMode.dark,
      home: const DefaultTextStyle(
        style: TextStyle(
          fontSize: 14,
        ),
        child: Scaffold(body: MainViewWrapper()),
      ));
}
