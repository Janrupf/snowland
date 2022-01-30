import 'package:flutter/material.dart';
import 'package:snowland_control_panel/data/property.dart';

class NumberPropertyEditor<T extends num> extends StatefulWidget {
  final bool signed;
  final bool decimal;
  final bool draggable;
  final ConfigurationProperty<T> property;

  const NumberPropertyEditor({
    Key? key,
    required this.property,
    this.signed = false,
    this.draggable = false,
  })  : decimal = T == double,
        super(key: key);

  @override
  State<StatefulWidget> createState() => _NumberPropertyEditorState<T>();
}

final _charZero = '0'.codeUnitAt(0);
final _charNine = '9'.codeUnitAt(0);
final _charMinus = '-'.codeUnitAt(0);
final _charDot = '.'.codeUnitAt(0);

class _NumberPropertyEditorState<T extends num>
    extends State<NumberPropertyEditor<T>> {
  late T _value;
  late final TextEditingController _controller;

  @override
  void initState() {
    super.initState();

    _value = widget.property.obtain(context);
    _controller = TextEditingController(text: _value.toString());
  }

  @override
  Widget build(BuildContext context) => _maybeDraggable(
        TextField(
          controller: _controller,
          keyboardType: TextInputType.numberWithOptions(
            decimal: widget.decimal,
            signed: widget.signed,
          ),
          onChanged: (newValue) => setState(() {
            int invalid = _findInvalidChar(newValue);

            if (invalid != -1) {
              newValue = newValue.substring(0, invalid) +
                  newValue.substring(invalid + 1);
              _controller.text = newValue;
              _controller.selection = TextSelection.collapsed(offset: invalid);
            }

            if (newValue.isEmpty) {
              return;
            }

            if (newValue.startsWith(".") && widget.decimal) {
              newValue = "0" + newValue;
              _controller.text = newValue;
              _controller.selection =
                  TextSelection.collapsed(offset: newValue.length);
            }

            final T v;
            if (T == int) {
              v = int.parse(newValue) as T;
            } else {
              v = double.parse(newValue) as T;
            }

            _value = v;

            widget.property.set(context, v);
          }),
        ),
      );

  void _updateValue(T v, {bool updateController = false}) {
    _value = v;
    widget.property.set(context, v);

    if (updateController) {
      final str = v.toString();

      _controller.text = str;
      _controller.selection = TextSelection.collapsed(offset: str.length);
    }
  }

  Widget _maybeDraggable(Widget child) {
    if (widget.draggable) {
      return _applyDraggableWrapper(child);
    } else {
      return child;
    }
  }

  int _findInvalidChar(String numberInput) {
    bool hasDot = false;

    for (int i = 0; i < numberInput.length; i++) {
      int c = numberInput.codeUnitAt(i);

      if (c == _charDot) {
        if (hasDot || !widget.decimal) {
          return i;
        }

        hasDot = true;
      } else if (c == _charMinus) {
        if (i != 0 || !widget.signed) {
          return i;
        }
      } else if (c < _charZero || c > _charNine) {
        return i;
      }
    }

    return -1;
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

      if(v != 0) {
        setState(() {
          _updateValue((_value + v) as T, updateController: true);
        });
      }
    }
  }
}
