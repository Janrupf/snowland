import 'package:snowland_control_panel/ffi/control_panel_api.dart';

/// Simple helper for logging.
///
/// This logger plugs into the rust logger and dispatches logging calls to the
/// rust interface.
class Logger {
  /// The component to log with, usually the name of the
  /// current file without .dart.
  final String component;

  const Logger(this.component);

  /// Logs [message] at trace level.
  void trace(String message) {
    ControlPanelAPI.instance.log(component, "trace", message);
  }

  /// Logs [message] at debug level.
  void debug(String message) {
    ControlPanelAPI.instance.log(component, "debug", message);
  }

  /// Logs [message] at info level.
  void info(String message) {
    ControlPanelAPI.instance.log(component, "info", message);
  }

  /// Logs [message] at warn level.
  void warn(String message) {
    ControlPanelAPI.instance.log(component, "warn", message);
  }

  /// Logs [message] at error level.
  void error(String message) {
    ControlPanelAPI.instance.log(component, "error", message);
  }
}
