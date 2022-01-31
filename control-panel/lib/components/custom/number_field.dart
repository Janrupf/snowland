import 'package:flutter/material.dart';

class NumberField<T extends num> extends StatefulWidget {
  final T initialValue;
  final ValueChanged<T> onChanged;
  final TextEditingController? controller;
  final bool decimal;
  final bool signed;

  const NumberField({
    Key? key,
    required this.initialValue,
    required this.onChanged,
    this.controller,
    this.signed = false,
  }) : decimal = T == double, super(key: key);

  @override
  State<StatefulWidget> createState() => _NumberFieldState<T>();
}

final _charZero = '0'.codeUnitAt(0);
final _charNine = '9'.codeUnitAt(0);
final _charMinus = '-'.codeUnitAt(0);
final _charDot = '.'.codeUnitAt(0);

class _NumberFieldState<T extends num> extends State<NumberField<T>> {
  late final TextEditingController _controller;
  
  @override
  void initState() {
    super.initState();
    _controller = widget.controller ?? TextEditingController(text: widget.initialValue.toString());
  }
  
  @override
  Widget build(BuildContext context) => TextField(
        controller: _controller,
        keyboardType: TextInputType.numberWithOptions(
          decimal: widget.decimal,
          signed: widget.signed,
        ),
        onChanged: (newValue) {
          int invalid = _findInvalidChar(newValue);

          if (invalid != -1) {
            newValue = newValue.substring(0, invalid) +
                newValue.substring(invalid + 1);
            _controller.text = newValue;
            _controller.selection = TextSelection.collapsed(offset: invalid);
          }

          if (newValue.isEmpty || newValue == "-") {
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

          _updateValue(v);
        },
      );

  void _updateValue(T v, {bool updateController = false}) {
    if (updateController) {
      final str = v.toString();

      _controller.text = str;
      _controller.selection = TextSelection.collapsed(offset: str.length);
    }

    widget.onChanged(v);
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
}
