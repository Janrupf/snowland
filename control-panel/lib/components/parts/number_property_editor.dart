import 'package:flutter/material.dart';
import 'package:snowland_control_panel/data/property.dart';

class NumberPropertyEditor extends StatefulWidget {
  final bool signed;
  final bool decimal;
  final ConfigurationProperty<double> property;

  const NumberPropertyEditor({
    Key? key,
    required this.property,
    this.signed = false,
    this.decimal = true,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => _NumberPropertyEditorState();
}

final _charZero = '0'.codeUnitAt(0);
final _charNine = '9'.codeUnitAt(0);
final _charMinus = '-'.codeUnitAt(0);
final _charDot = '.'.codeUnitAt(0);

class _NumberPropertyEditorState extends State<NumberPropertyEditor> {
  late double _value;
  late final TextEditingController _controller;

  @override
  void initState() {
    super.initState();
    _value = widget.property.obtain(context);
    _controller = TextEditingController(text: _value.toString());
  }

  @override
  Widget build(BuildContext context) => TextField(
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
            _controller.selection =
                TextSelection.collapsed(offset: invalid);
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

          final v = double.parse(newValue);
          _value = v;

          widget.property.set(context, v);
        }),
      );

  int _findInvalidChar(String numberInput) {
    bool hasDot = false;

    for (int i = 0; i < numberInput.length; i++) {
      int c = numberInput.codeUnitAt(i);

      if (c == _charDot) {
        if (hasDot) {
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
