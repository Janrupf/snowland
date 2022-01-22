import 'dart:collection';

class IPCDataHelper {
  IPCDataHelper._();

  /// Tries to cast the [value] to another one, throwing an [ArgumentError] if
  /// the value can't be casted.
  static T as<T>(dynamic value, String name) {
    if(value !is T) {
      throw ArgumentError.value(value, name, "$name is not a $T");
    }

    return value;
  }

  /// Tries to access a property with a given [name] by interpreting this
  /// as a [LinkedHashMap].
  static T property<T>(dynamic value, String name) {
    final self = as<LinkedHashMap>(value, name);
    final propertyValue = self[name];

    if(propertyValue == null && !isNullable<T>()) {
      throw ArgumentError.notNull(name);
    }

    return as<T>(propertyValue, name);
  }

  /// Determines whether a [T] is nullable.
  static bool isNullable<T>() => null is T;
}

