import 'package:flutter/widgets.dart';
import 'package:snowland_control_panel/data/ipc_data_ext.dart';

typedef ConfigurationChangeCallback = void Function();

class ConfigurationProvider extends StatefulWidget {
  final Widget child;
  final Map configuration;
  final ConfigurationChangeCallback onChange;

  const ConfigurationProvider({
    Key? key,
    required this.child,
    required this.configuration,
    required this.onChange,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => ConfigurationProviderState();

  static ConfigurationProviderState of(BuildContext context) {
    final state = context.findAncestorStateOfType<ConfigurationProviderState>();
    assert(state != null, "No parent ConfigurationProvider found");
    return state!;
  }
}

class ConfigurationProviderState extends State<ConfigurationProvider> {
  @override
  Widget build(BuildContext context) => widget.child;

  dynamic lookupProperty<T>(List<String> path) {
    assert(path.isNotEmpty, "property path must contain at least one property");
    return _recursiveLookup(widget.configuration, path[0], path.sublist(1));
  }

  dynamic updateProperty(List<String> path, dynamic newValue) {
    assert(path.isNotEmpty, "property path must contain at least one property");

    final dynamic container;
    final String targetProperty;

    if (path.length == 1) {
      container = widget.configuration;
      targetProperty = path[0];
    } else {
      container = _recursiveLookup(
        widget.configuration,
        path[0],
        path.sublist(1, path.length - 1),
      );

      targetProperty = path.last;
    }

    if (container is! Map) {
      throw BadPropertyPathException(targetProperty);
    }

    container[targetProperty] = newValue;
    widget.onChange();
  }

  dynamic _recursiveLookup<T>(
    dynamic obj,
    String property,
    List<String> remaining,
  ) {
    if (obj == null) {
      return null;
    } else if (obj is! Map) {
      throw BadPropertyPathException(property);
    }

    if (remaining.isEmpty) {
      return obj[property];
    } else {
      return _recursiveLookup(
        obj[property],
        remaining[0],
        remaining.sublist(1),
      );
    }
  }
}

class ConfigurationPropertyGroup {
  final List<String> path;

  const ConfigurationPropertyGroup(this.path);

  ConfigurationProperty<T> property<T>(List<String> subPath) {
    return ConfigurationProperty(path + subPath);
  }

  ConfigurationPropertyList<T> listProperty<T>(List<String> subPath) {
    return ConfigurationPropertyList(path + subPath);
  }

  ConfigurationPropertyGroup group(List<String> subPath) {
    return ConfigurationPropertyGroup(path + subPath);
  }

  ConfigurationProperty<T> self<T>() {
    return property([]);
  }

  ConfigurationPropertyList<T> listSelf<T>() {
    return listProperty([]);
  }
}

class ConfigurationProperty<T> {
  final List<String> path;

  const ConfigurationProperty(this.path);

  T obtain(BuildContext context) {
    final value = ConfigurationProvider.of(context).lookupProperty(path);

    if (value is! T) {
      if (value == null) {
        throw BadPropertyTypeException(path, T, Null);
      } else {
        throw BadPropertyTypeException(path, T, value.runtimeType);
      }
    }

    return value;
  }

  T? obtainWhenValid(
    BuildContext context, {
    bool? allowNull
  }) {
    final realAllowNull = allowNull ?? null is T;

    final value = ConfigurationProvider.of(context).lookupProperty(path);
    if(!realAllowNull && value == null) {
      throw BadPropertyTypeException(path, T, Null);
    }

    return value is T ? value : null;
  }

  void set(BuildContext context, T newValue) {
    ConfigurationProvider.of(context).updateProperty(path, newValue);
  }
}

class ConfigurationPropertyList<T> {
  final ConfigurationProperty<List> _underlying;

  ConfigurationPropertyList(List<String> path)
      : _underlying = ConfigurationProperty(path);

  List<String> get path => _underlying.path;

  List<T> obtain(BuildContext context) {
    final value = _underlying.obtain(context);

    return value.map((v) => v as T).toList();
  }

  void set(BuildContext context, List<T> newValue) {
    _underlying.set(context, newValue.map((v) => v as dynamic).toList());
  }
}

class BadPropertyPathException implements Exception {
  final String failed;

  const BadPropertyPathException(this.failed);

  @override
  String toString() {
    return "encountered object which was not a map while lookup up property "
        "element $failed";
  }
}

class BadPropertyTypeException implements Exception {
  final List<String> path;
  final Type expected;
  final Type found;

  const BadPropertyTypeException(this.path, this.expected, this.found);

  @override
  String toString() {
    return "property ${path.join(", ")} was expected to be of $expected, but was of $found";
  }
}
