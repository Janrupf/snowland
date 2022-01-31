import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:snowland_control_panel/data/property.dart';
import 'package:snowland_control_panel/util/util.dart';

class NumberRangePropertyEditor<T extends num> extends StatefulWidget {
  final T min;
  final T max;
  final ConfigurationProperty<T> minProperty;
  final ConfigurationProperty<T> maxProperty;
  final T? step;

  const NumberRangePropertyEditor(
      {Key? key,
      required this.min,
      required this.max,
      required this.minProperty,
      required this.maxProperty,
      this.step})
      : super(key: key);

  @override
  State<StatefulWidget> createState() => _NumberRangePropertyEditorState<T>();
}

class _NumberRangePropertyEditorState<T extends num>
    extends State<NumberRangePropertyEditor<T>> {
  late RangeValues _current;

  @override
  void initState() {
    super.initState();
    final min = widget.minProperty.obtain(context);
    final max = widget.maxProperty.obtain(context);

    _current = RangeValues(
      math.max(min, widget.min).toDouble(),
      math.min(max, widget.max).toDouble(),
    );
  }

  @override
  Widget build(BuildContext context) => Row(
        mainAxisSize: MainAxisSize.max,
        children: [
          Text(Util.formatNumber(_current.start)),
          Expanded(
            child: RangeSlider(
              values: _current,
              min: widget.min.toDouble(),
              max: widget.max.toDouble(),
              divisions: Util.divisionForSteps(
                widget.min,
                widget.max,
                widget.step,
              ),
              onChanged: _onValueChanged,
            ),
          ),
          Text(Util.formatNumber(_current.end)),
        ],
      );

  void _onValueChanged(RangeValues newValues) {
    setState(() {
      _current = newValues;
    });

    if (T == int) {
      widget.minProperty.set(context, newValues.start.toInt() as T);
      widget.maxProperty.set(context, newValues.end.toInt() as T);
    } else {
      widget.minProperty.set(context, newValues.start.toDouble() as T);
      widget.maxProperty.set(context, newValues.end.toDouble() as T);
    }
  }
}
