class Util {
  const Util._();

  static int? divisionForSteps<T extends num>(T min, T max, T? steps) {
    if (steps == null) {
      if (T == int) {
        return (max - min).abs().toInt();
      } else {
        return null;
      }
    }

    final diff = (max - min).abs();
    return (diff / steps).floor();
  }

  static String formatNumber<T extends num>(T n) {
    if(T == int) {
      return n.toInt().toString();
    } else {
      return n.toDouble().toStringAsFixed(3).padLeft(7, " ");
    }
  }
}
