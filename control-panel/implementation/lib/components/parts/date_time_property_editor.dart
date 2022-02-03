import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:snowland_control_panel/components/custom/time_picker.dart';
import 'package:snowland_control_panel/data/property.dart';

class DateTimePropertyEditor extends StatefulWidget {
  final ConfigurationProperty<int> property;

  const DateTimePropertyEditor({
    Key? key,
    required this.property,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => _DateTimePropertyEditorState();
}

class _DateTimePropertyEditorState extends State<DateTimePropertyEditor> {
  late DateTime _current;

  @override
  void initState() {
    super.initState();
    _current = DateTime.fromMillisecondsSinceEpoch(
            widget.property.obtain(context),
            isUtc: true)
        .toLocal();
  }

  @override
  Widget build(BuildContext context) => ElevatedButton(
        onPressed: () => _onButtonPressed(context),
        child: Text(_formatDateTime()),
      );

  void _onButtonPressed(BuildContext context) {
    showDialog<DateTime>(
      context: context,
      builder: (context) => _DateTimeAlertDialog(initial: _current),
    ).then((value) {
      if (value != null && mounted) {
        setState(() {
          _current = value;
        });

        widget.property.set(context, _current.toUtc().millisecondsSinceEpoch);
      }
    });
  }

  String _formatDateTime() {
    // TODO: Initialize date formatting for other locales...
    return Intl().date().format(_current);
  }
}

class _DateTimeAlertDialog extends StatefulWidget {
  final DateTime initial;

  const _DateTimeAlertDialog({
    Key? key,
    required this.initial,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => _DateTimeAlertDialogState();
}

class _DateTimeAlertDialogState extends State<_DateTimeAlertDialog> {
  late DateTime _current;
  bool _error = false;

  @override
  void initState() {
    super.initState();
    _current = widget.initial;
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text("Choose a new date and time"),
      content: SingleChildScrollView(
        child: Wrap(
          alignment: WrapAlignment.center,
          children: [
            SizedBox(
              height: 300,
              width: 300,
              child: CalendarDatePicker(
                  initialDate: _current,
                  firstDate:
                      DateTime.fromMillisecondsSinceEpoch(0, isUtc: true),
                  lastDate: DateTime.now().toUtc().add(
                        const Duration(
                            days: 365 * 100 /* this is probably ok */),
                      ),
                  onDateChanged: _onChanged),
            ),
            const SizedBox(
              width: 20,
            ),
            SizedBox(
                height: 200,
                width: 300,
                child: TimePickerInput(
                  initialSelectedTime: TimeOfDay.fromDateTime(_current),
                  onChanged: (v) {
                    if (v != null) {
                      _setError(false);

                      final clone = DateTime(
                          _current.year,
                          _current.month,
                          _current.day,
                          v.hour,
                          v.minute,
                          _current.second,
                          _current.millisecond,
                          _current.microsecond);

                      _onChanged(clone);
                    } else {
                      _setError(true);
                    }
                  },
                ))
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: Navigator.of(context).pop,
          child: const Text("Cancel"),
        ),
        TextButton(
          onPressed: _error ? null : () => _submit(context),
          child: const Text("Submit"),
        ),
      ],
    );
  }

  void _onChanged(DateTime newDateTime) => setState(() {
        _current = newDateTime;
      });

  void _setError(bool error) {
    if (_error != error) {
      setState(() {
        _error = error;
      });
    }
  }

  void _submit(BuildContext context) {
    Navigator.of(context).pop(_current);
  }
}
