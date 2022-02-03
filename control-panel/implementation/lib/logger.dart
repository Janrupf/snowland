import 'package:snowland_control_panel/com/dart_to_native.dart';

/// Simple helper for logging.
///
/// This logger plugs into the rust logger and dispatches logging calls to the
/// rust interface.
class Logger {
  /// The component to log with, usually the name current file without .dart.
  final String component;

  const Logger(this.component);

  /// Logs [message] at trace level.
  void trace(String message) {
    DartToNativeCommunicator.instance.log(component, "Trace", message);
  }

  /// Logs [message] at debug level.
  void debug(String message) {
    DartToNativeCommunicator.instance.log(component, "Debug", message);
  }

  /// Logs [message] at info level.
  void info(String message) {
    DartToNativeCommunicator.instance.log(component, "Info", message);
  }

  /// Logs [message] at warn level.
  void warn(String message) {
    DartToNativeCommunicator.instance.log(component, "Warn", message);
  }

  /// Logs [message] at error level.
  void error(String message) {
    DartToNativeCommunicator.instance.log(component, "Error", message);
  }
}
