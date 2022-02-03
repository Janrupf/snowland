import 'package:flutter/material.dart';
import 'package:snowland_control_panel/data/property.dart';

enum _VerticalAlignment { top, center, bottom }

enum _HorizontalAlignment { left, center, right }

class _Alignments {
  static _SelectableAlignment<_HorizontalAlignment> horizontal() =>
      _SelectableAlignment(
        values: {
          _HorizontalAlignment.left: "Left",
          _HorizontalAlignment.center: "Center",
          _HorizontalAlignment.right: "Right"
        },
      );

  static _SelectableAlignment<_VerticalAlignment> vertical() =>
      _SelectableAlignment(
        values: {
          _VerticalAlignment.top: "Top",
          _VerticalAlignment.center: "Center",
          _VerticalAlignment.bottom: "Bottom",
        },
      );
}

class _SelectableAlignment<T> {
  final Map<String, T> backMapping;
  final Map<T, String> values;

  _SelectableAlignment({required this.values})
      : backMapping = values.map((k, v) => MapEntry(v, k));

  T fromProperty(BuildContext context, ConfigurationProperty<String> property) {
    final ident = property.obtain(context);
    final value = backMapping[ident];

    if (value == null) {
      throw ArgumentError.value(
        ident,
        property.path.join(", "),
        "$ident is not a valid $T",
      );
    } else {
      return value;
    }
  }

  void writeToProperty(
    T selected,
    BuildContext context,
    ConfigurationProperty<String> property,
  ) {
    final ident = values[selected];

    if (ident == null) {
      throw ArgumentError.value(
        selected,
        "selected",
        "$selected is not mappable to a string for $T",
      );
    } else {
      property.set(context, ident);
    }
  }
}

enum AlignmentPropertyType { horizontal, vertical }

class AlignmentPropertyEditor extends StatefulWidget {
  final ConfigurationProperty<String> property;
  final AlignmentPropertyType type;

  const AlignmentPropertyEditor({
    Key? key,
    required this.property,
    required this.type,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => _AlignmentPropertyEditorState();
}

class _AlignmentPropertyEditorState extends State<AlignmentPropertyEditor> {
  late final _SelectableAlignment _alignment;
  late dynamic _current;

  @override
  void initState() {
    super.initState();
    switch (widget.type) {
      case AlignmentPropertyType.horizontal:
        _alignment = _Alignments.horizontal();
        break;

      case AlignmentPropertyType.vertical:
        _alignment = _Alignments.vertical();
        break;
    }

    _current = _alignment.fromProperty(context, widget.property);
  }

  @override
  Widget build(BuildContext context) => DropdownButton(
        items: _items(context),
        value: _current,
        onChanged: _onChanged,
      );

  List<DropdownMenuItem> _items(BuildContext context) =>
      _alignment.values.entries
          .map((e) => DropdownMenuItem(
              key: Key(e.value), child: Text(e.value), value: e.key))
          .toList();

  void _onChanged(dynamic newValue) => setState(() {
        _current = newValue;
        _alignment.writeToProperty(_current, context, widget.property);
      });
}
