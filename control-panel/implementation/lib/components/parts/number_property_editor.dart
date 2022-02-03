import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/custom/number_field.dart';
import 'package:snowland_control_panel/data/property.dart';

class NumberPropertyEditor<T extends num> extends StatefulWidget {
  final bool signed;
  final bool draggable;
  final ConfigurationProperty<T> property;

  const NumberPropertyEditor({
    Key? key,
    required this.property,
    this.signed = false,
    this.draggable = false,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => _NumberPropertyEditorState<T>();
}

class _NumberPropertyEditorState<T extends num>
    extends State<NumberPropertyEditor<T>> {
  late T _value;

  @override
  void initState() {
    super.initState();

    _value = widget.property.obtain(context);
  }

  @override
  Widget build(BuildContext context) => _maybeDraggable(
        NumberField<T>(
          initialValue: _value,
          signed: widget.signed,
          onChanged: (v) => setState(() {
            widget.property.set(context, v);
            _value = v;
          }),
        ),
      );

  Widget _maybeDraggable(Widget child) {
    if (widget.draggable) {
      return _applyDraggableWrapper(child);
    } else {
      return child;
    }
  }

  Widget _applyDraggableWrapper(Widget child) => GestureDetector(
        onHorizontalDragStart: (d) {},
        onHorizontalDragUpdate: _onDragUpdate,
        child: child,
      );

  void _onDragUpdate(DragUpdateDetails details) {
    if (details.primaryDelta != null) {
      final T v;
      if (T == int) {
        v = details.primaryDelta!.toInt() as T;
      } else {
        v = details.primaryDelta! as T;
      }

      if (v != 0) {
        setState(() {
          _value = (_value + v) as T;
        });
      }
    }
  }
}
