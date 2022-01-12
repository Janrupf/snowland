import 'dart:collection';

/// Represents the entire snowland daemon configuration, including installed
/// [modules].
class Configuration {
  final List<InstalledModule> modules;

  const Configuration._(this.modules);

  /// Converts the native [data] into a dart instance of the [Configuration].
  factory Configuration.fromData(dynamic data) {
    if (data is! LinkedHashMap) {
      throw ArgumentError.value(data, "data", "Not a LinkedHashMap");
    }

    final d = data;
    final modules = d["modules"];

    if (modules is! List) {
      throw ArgumentError.value(modules, "data.modules", "Not a List");
    }

    final installed =
        modules.map((m) => InstalledModule.fromData(m)).toList(growable: false);

    return Configuration._(installed);
  }
}

/// A module which is currently installed on the daemon.
class InstalledModule {
  /// The type of the module
  final String type;

  const InstalledModule._(this.type);

  factory InstalledModule.fromData(dynamic data) {
    if (data is! LinkedHashMap) {
      throw ArgumentError.value(data, "data", "Not a LinkedHashMap");
    }

    return InstalledModule._(data["ty"]);
  }
}
