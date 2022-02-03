import 'package:snowland_control_panel/data/ipc_data_ext.dart';

/// Represents the entire snowland daemon configuration, including installed
/// [modules].
class Configuration {
  final List<InstalledModule> modules;

  const Configuration._(this.modules);

  /// Converts the native [data] into a dart instance of the [Configuration].
  factory Configuration.fromData(dynamic data) {
    final modules = IPCDataHelper.property<List>(data, "modules");
    final installed = modules.map((m) => InstalledModule.fromData(m)).toList();

    return Configuration._(installed);
  }
}

/// A module which is currently installed on the daemon.
class InstalledModule {
  /// The type of the module
  final String type;

  /// The configuration structure of the module
  final Map configuration;

  const InstalledModule._(this.type, this.configuration);

  factory InstalledModule.fromData(dynamic data) {
    return InstalledModule._(
      IPCDataHelper.property<String>(data, "ty"),
      IPCDataHelper.property<Map>(data, "configuration"),
    );
  }
}
