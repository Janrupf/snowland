import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:snowland_control_panel/data/display.dart';

/// Widget for selecting a certain display.
class DisplaySelector extends LeafRenderObjectWidget {
  /// The currently active display selection.
  final DisplaySelection selection;

  /// The list of displays available to be selected.
  final List<Display> displays;

  final ThemeData? theme;

  const DisplaySelector({
    Key? key,
    required this.selection,
    required this.displays,
    this.theme,
  }) : super(key: key);

  @override
  RenderObject createRenderObject(BuildContext context) => DisplayRenderObject(
        selection: selection,
        displays: displays,
        theme: _selectTheme(context),
      );

  @override
  void updateRenderObject(
    BuildContext context,
    DisplayRenderObject renderObject,
  ) {
    renderObject
      ..selection = selection
      ..displays = displays
      ..theme = _selectTheme(context);
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties.add(
      StringProperty("selection", selection.displayName(), quoted: false),
    );
    properties.add(IterableProperty("displays", displays));
  }

  ThemeData _selectTheme(BuildContext context) => theme ?? Theme.of(context);
}

class DisplayRenderObject extends RenderBox {
  static const margin = 40.0;

  DisplaySelection _selection;

  DisplaySelection get selection => _selection;

  set selection(DisplaySelection newSelection) {
    if (_selection != newSelection) {
      _selection = newSelection;
      markNeedsLayout();
      markNeedsPaint();
      markNeedsSemanticsUpdate();
    }
  }

  List<Display> _displays;

  List<Display> get displays => _displays;

  set displays(List<Display> newDisplays) {
    if (_displays != newDisplays) {
      _displays = newDisplays;
      markNeedsLayout();
      markNeedsPaint();
      markNeedsSemanticsUpdate();
    }
  }

  ThemeData _theme;

  ThemeData get theme => _theme;

  set theme(ThemeData newTheme) {
    if (_theme != newTheme) {
      _theme = newTheme;
      markNeedsPaint();
    }
  }

  final List<_SelectableDisplay> _displaysOnCanvas;

  DisplayRenderObject({
    required DisplaySelection selection,
    required List<Display> displays,
    required ThemeData theme,
  })  : _selection = selection,
        _displays = displays,
        _theme = theme,
        _displaysOnCanvas = [];

  @override
  void performResize() {
    size = computeDryLayout(constraints);
    assert(size.isFinite);

    _displaysOnCanvas.clear();

    final displaysArea = _findUsedSize();
    final workSize = Size(size.width - margin * 2, size.height - margin * 2);

    double targetAspectRatio = math.min(
      1,
      math.min(
        workSize.width / displaysArea.width,
        workSize.height / displaysArea.height,
      ),
    );

    final centerHeight = size.height / 2;
    final drawStartHeight =
        centerHeight - (displaysArea.height * targetAspectRatio) / 2;

    for (final display in displays) {
      final displayRect = Rect.fromLTWH(
        (display.x.toDouble() * targetAspectRatio) + margin,
        (display.y.toDouble() * targetAspectRatio) + drawStartHeight,
        display.width.toDouble() * targetAspectRatio,
        display.height.toDouble() * targetAspectRatio,
      );

      final String name;
      if (display.primary) {
        name = "${display.name}\n(primary)";
      } else {
        name = display.name;
      }

      final nameSpan = TextSpan(
          text: name,
          style: TextStyle(
            color: theme.buttonTheme.colorScheme?.onBackground ?? Colors.black,
          ));

      final namePainter = TextPainter(
          text: nameSpan,
          textAlign: TextAlign.center,
          textDirection: TextDirection.ltr);

      namePainter.layout(maxWidth: displayRect.width - 20);

      final nameOffset = displayRect.center
          .translate(-(namePainter.width / 2), -(namePainter.height / 2));

      _displaysOnCanvas.add(
        _SelectableDisplay(
          area: displayRect,
          namePainter: namePainter,
          nameOffset: nameOffset,
          display: display,
        ),
      );
    }
  }

  @override
  Size computeDryLayout(BoxConstraints constraints) {
    return constraints.biggest;
  }

  // We always take the size we can get
  @override
  bool get sizedByParent => true;

  @override
  void paint(PaintingContext context, Offset offset) {
    final canvas = context.canvas;
    canvas.save();

    canvas.translate(offset.dx, offset.dy);

    final widgetRect = RRect.fromLTRBXY(0, 0, size.width, size.height, 20, 20);

    final backgroundPaint = Paint()
      ..color = theme.backgroundColor
      ..style = PaintingStyle.fill;

    final borderPaint = Paint()
      ..color = theme.highlightColor
      ..style = PaintingStyle.stroke;

    final displayBackgroundPaint = Paint()
      ..color = theme.buttonTheme.colorScheme?.background ?? Colors.white
      ..style = PaintingStyle.fill;

    final selectedDisplayBackgroundPaint = Paint()
      ..color = theme.buttonTheme.colorScheme?.primary ?? Colors.blue
      ..style = PaintingStyle.fill;

    final displayBorderPaint = Paint()
      ..color = theme.buttonTheme.colorScheme?.shadow ?? Colors.red
      ..strokeWidth = 4.0
      ..style = PaintingStyle.stroke;

    canvas.drawRRect(widgetRect, backgroundPaint);
    canvas.drawRRect(widgetRect, borderPaint);

    for (final selectable in _displaysOnCanvas) {
      canvas.drawRect(
          selectable.area,
          _selection.matches(selectable.display)
              ? selectedDisplayBackgroundPaint
              : displayBackgroundPaint);
      canvas.drawRect(selectable.area, displayBorderPaint);

      selectable.namePainter.paint(canvas, selectable.nameOffset);
    }

    canvas.restore();
  }

  /// Finds the size required by the displays.
  Size _findUsedSize() {
    int? leftBorder;
    int? rightBorder;

    int? upperBorder;
    int? lowerBorder;

    for (final display in displays) {
      final left = display.x;
      final right = display.x + display.width;

      final upper = display.y;
      final lower = display.y + display.height;

      if (leftBorder == null || left < leftBorder) {
        leftBorder = left;
      }

      if (rightBorder == null || right > rightBorder) {
        rightBorder = right;
      }

      if (upperBorder == null || upper < upperBorder) {
        upperBorder = upper;
      }

      if (lowerBorder == null || lower > lowerBorder) {
        lowerBorder = lower;
      }
    }

    assert(leftBorder != null);
    assert(rightBorder != null);
    assert(upperBorder != null);
    assert(lowerBorder != null);

    return Size(
      (leftBorder! - rightBorder!).abs().toDouble(),
      (upperBorder! - lowerBorder!).abs().toDouble(),
    );
  }
}

@immutable
class _SelectableDisplay {
  final Rect area;
  final TextPainter namePainter;
  final Offset nameOffset;
  final Display display;

  const _SelectableDisplay({
    required this.area,
    required this.namePainter,
    required this.nameOffset,
    required this.display,
  });
}
