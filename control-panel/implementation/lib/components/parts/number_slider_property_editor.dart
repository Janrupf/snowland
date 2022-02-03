import 'package:flutter/material.dart';
import 'package:snowland_control_panel/data/property.dart';
import 'package:snowland_control_panel/util/util.dart';

class NumberSliderPropertyEditor<T extends num> extends StatefulWidget {
  final T min;
  final T max;
  final ConfigurationProperty<T> property;
  final T? step;

  const NumberSliderPropertyEditor({
    Key? key,
    required this.min,
    required this.max,
    required this.property,
    this.step,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => _NumberSliderPropertyEditorState<T>();
}

class _NumberSliderPropertyEditorState<T extends num>
    extends State<NumberSliderPropertyEditor<T>> {
  late T _current;

  @override
  void initState() {
    super.initState();
    _current = widget.property.obtain(context);
  }

  @override
  Widget build(BuildContext context) => Row(
        children: [
          Text(Util.formatNumber(_current)),
          Expanded(
            child: Slider(
              min: widget.min.toDouble(),
              max: widget.max.toDouble(),
              value: _current.toDouble(),
              divisions: Util.divisionForSteps(
                widget.min,
                widget.max,
                widget.step,
              ),
              onChanged: _onChanged,
            ),
          ),
        ],
      );

  void _onChanged(double newValue) {
    if (T == int) {
      setState(() {
        _current = newValue.toInt() as T;
      });

      widget.property.set(context, newValue.toInt() as T);
    } else {
      setState(() {
        _current = newValue.toDouble() as T;
      });

      widget.property.set(context, newValue.toDouble() as T);
    }
  }
}
