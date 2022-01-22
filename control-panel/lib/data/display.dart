import 'package:flutter/widgets.dart';
import 'package:snowland_control_panel/data/ipc_data_ext.dart';

class Display {
  static List<Display> fromDataList(dynamic list) {
    final casted = IPCDataHelper.as<List>(list, "list");
    return casted.map((entry) => Display.fromData(entry)).toList();
  }

  factory Display.fromData(dynamic data) => Display(
        name: IPCDataHelper.property<String>(data, "name"),
        id: IPCDataHelper.property<String>(data, "id"),
        x: IPCDataHelper.property<int>(data, "x"),
        y: IPCDataHelper.property<int>(data, "y"),
        width: IPCDataHelper.property<int>(data, "width"),
        height: IPCDataHelper.property<int>(data, "height"),
        primary: IPCDataHelper.property<bool>(data, "primary"),
      );

  /// The user friendly name of the display.
  final String name;

  /// The unique identifier of the display.
  ///
  /// Displays with the same [id] are considered to be the same display,
  /// regardless of their other properties. Usually [width] and [height] will
  /// not change for a certain [id], but technically they could.
  final String id;

  /// The [x] offset of the display on the virtual rectangle, always positive.
  final int x;

  /// The [y] offset of the display on the virtual rectangle, always positive.
  final int y;

  /// The [width] of the display, always positive (I hope...).
  final int width;

  /// The [height] of the display, always positive (I hope...).
  final int height;

  /// Whether this display is the [primary] display on the system, should
  /// always only be set for one display instance on one snowland daemon.
  final bool primary;

  const Display({
    required this.name,
    required this.id,
    required this.x,
    required this.y,
    required this.width,
    required this.height,
    required this.primary,
  });

  @override
  bool operator ==(Object other) {
    // We only really care about the id
    return other is Display && other.id == id;
  }

  @override
  int get hashCode =>
      // Mix in the hash of the type to have something to distinguish Display
      // instances from instances of String
      hashValues(Display, id);
}
