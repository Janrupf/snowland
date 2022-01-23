import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:snowland_control_panel/data/display.dart';

typedef DisplaySelectionCallback = void Function(DisplaySelection newSelection);

/// Widget for selecting a certain display.
class DisplaySelector extends LeafRenderObjectWidget {
  /// The currently active display selection.
  final DisplaySelection selection;

  /// The list of displays available to be selected.
  final List<Display> displays;

  /// The callback to invoke when the selection changed.
  final DisplaySelectionCallback onChanged;

  /// The theme to use for the selector.
  final ThemeData? theme;

  const DisplaySelector({
    Key? key,
    required this.selection,
    required this.displays,
    required this.onChanged,
    this.theme,
  }) : super(key: key);

  @override
  RenderObject createRenderObject(BuildContext context) => DisplayRenderObject(
        selection: selection,
        displays: displays,
        theme: _selectTheme(context),
        onChanged: onChanged,
      );

  @override
  void updateRenderObject(
    BuildContext context,
    DisplayRenderObject renderObject,
  ) {
    renderObject
      ..selection = selection
      ..displays = displays
      ..theme = _selectTheme(context)
      ..onChanged = onChanged;
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

  DisplaySelectionCallback onChanged;

  final List<_SelectableDisplay> _displaysOnCanvas;
  Display? _hoveredDisplay;
  Display? _pendingSelectionDisplay;

  DisplayRenderObject({
    required DisplaySelection selection,
    required List<Display> displays,
    required ThemeData theme,
    required this.onChanged,
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

      final nameSpan = _buildDisplaySpan(display);

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
  bool hitTestSelf(Offset position) => true;

  @override
  void handleEvent(PointerEvent event, covariant BoxHitTestEntry entry) {
    assert(debugHandleEvent(event, entry));

    if (event is PointerHoverEvent) {
      Display? newHoveredDisplay = _findDisplayAt(event.localPosition);
      if (_hoveredDisplay != newHoveredDisplay) {
        _hoveredDisplay = newHoveredDisplay;
        markNeedsPaint();
      }
    } else if (event is PointerDownEvent) {
      _pendingSelectionDisplay = _findDisplayAt(event.localPosition);
    } else if (event is PointerUpEvent) {
      Display? upDisplay = _findDisplayAt(event.localPosition);
      if (upDisplay != null && upDisplay == _pendingSelectionDisplay) {
        final newSelection = DisplaySelection.fromDisplay(upDisplay);

        if(newSelection != _selection) {
          onChanged(newSelection);
        }
      }

      _pendingSelectionDisplay = null;
    } else if(event is PointerExitEvent) {
      _hoveredDisplay = null;
      markNeedsPaint();
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

    final hoveredDisplayBackgroundPaint = Paint()
      ..color = theme.buttonTheme.colorScheme?.secondary ?? Colors.green
      ..style = PaintingStyle.fill;

    final displayBorderPaint = Paint()
      ..color = theme.buttonTheme.colorScheme?.shadow ?? Colors.red
      ..strokeWidth = 4.0
      ..style = PaintingStyle.stroke;

    canvas.drawRRect(widgetRect, backgroundPaint);
    canvas.drawRRect(widgetRect, borderPaint);

    for (final selectable in _displaysOnCanvas) {
      final Paint backgroundPaint;
      if (_selection.matches(selectable.display)) {
        backgroundPaint = selectedDisplayBackgroundPaint;
      } else if (selectable.display == _hoveredDisplay) {
        backgroundPaint = hoveredDisplayBackgroundPaint;
      } else {
        backgroundPaint = displayBackgroundPaint;
      }

      canvas.drawRect(selectable.area, backgroundPaint);
      canvas.drawRect(selectable.area, displayBorderPaint);

      selectable.namePainter.text = _buildDisplaySpan(selectable.display);
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

  Display? _findDisplayAt(Offset offset) {
    for (final selectable in _displaysOnCanvas) {
      if (selectable.area.contains(offset)) {
        return selectable.display;
      }
    }

    return null;
  }

  TextSpan _buildDisplaySpan(Display display) {
    final Color textColor;

    if (_selection.matches(display)) {
      textColor = theme.buttonTheme.colorScheme?.onPrimary ?? Colors.white;
    } else if (_hoveredDisplay == display) {
      textColor = theme.buttonTheme.colorScheme?.onSecondary ?? Colors.white;
    } else {
      textColor = theme.buttonTheme.colorScheme?.onBackground ?? Colors.black;
    }

    final String label;
    if (display.primary) {
      label = "${display.name}\n(primary)";
    } else {
      label = display.name;
    }

    return TextSpan(text: label, style: TextStyle(color: textColor));
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
