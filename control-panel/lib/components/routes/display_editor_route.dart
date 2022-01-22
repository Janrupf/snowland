import 'package:flutter/material.dart';
import 'package:snowland_control_panel/components/display_provider.dart';
import 'package:snowland_control_panel/data/display.dart';
import 'package:snowland_control_panel/logger.dart';

typedef DisplaySelectionCallback = void Function(DisplaySelection newSelection);

const Logger _logger = Logger("display_editor");

class DisplayEditorRoute extends StatelessWidget {
  final DisplaySelectionCallback onChanged;

  const DisplayEditorRoute({
    Key? key,
    required this.onChanged,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) => Material(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Center(
              child: ElevatedButton(
                child: const Text("Back"),
                onPressed: () => Navigator.of(context).pop(),
              ),
            ),
            DisplayProvider(
              loadingBuilder: _buildLoading,
              availableBuilder: _buildAvailable,
            )
          ],
        ),
      );

  Widget _buildLoading(BuildContext context) => const Expanded(
        child: Center(
          // TODO: A progress indicator seems to block the event loop sometimes
          child: Text("Waiting for daemon"),
        ),
      );

  Widget _buildAvailable(BuildContext context) =>
      _DisplayEditor(onChanged: onChanged);
}

class _DisplayEditor extends StatelessWidget {
  final DisplaySelectionCallback onChanged;

  const _DisplayEditor({Key? key, required this.onChanged}) : super(key: key);

  @override
  Widget build(BuildContext context) => Expanded(
        child: Padding(
          padding: const EdgeInsets.all(40),
          child: CustomPaint(
            painter: _DisplayEditorPainter(
              displays: DisplayProvider.of(context).displays,
              theme: Theme.of(context),
            ),
          ),
        ),
      );
}

class _DisplayEditorPainter extends CustomPainter {
  final List<Display> displays;
  final ThemeData theme;

  _DisplayEditorPainter({required this.displays, required this.theme});

  @override
  void paint(Canvas canvas, Size canvasSize) {
    if (displays.isEmpty) {
      // TODO: Write that no displays are available
      return;
    }

    final monitorArea = _findUsedSize();

    final roundedRect = RRect.fromLTRBXY(
      0,
      0,
      canvasSize.width,
      canvasSize.height,
      20,
      20,
    );

    final backgroundPaint = Paint()
      ..color = theme.backgroundColor
      ..style = PaintingStyle.fill;

    final borderPaint = Paint()
      ..color = theme.highlightColor
      ..style = PaintingStyle.stroke;

    canvas.drawRRect(roundedRect, backgroundPaint);
    canvas.drawRRect(roundedRect, borderPaint);

    const margin = 40.0;

    final workSize = Size(
      canvasSize.width - margin * 2,
      canvasSize.height - margin * 2,
    );

    double targetAspectRatio = 1;

    double widthRatio = workSize.width / monitorArea.width;
    if (widthRatio < targetAspectRatio) {
      targetAspectRatio = widthRatio;
    }

    double heightRatio = workSize.height / monitorArea.height;
    if (heightRatio < targetAspectRatio) {
      targetAspectRatio = heightRatio;
    }

    final centerHeight = canvasSize.height / 2;
    final drawStartHeight =
        centerHeight - (monitorArea.height * targetAspectRatio) / 2;

    canvas.translate(margin, drawStartHeight);

    for (final display in displays) {
      _drawDisplay(canvas, workSize, display, targetAspectRatio);
    }
  }

  void _drawDisplay(
      Canvas canvas, Size workSize, Display display, double aspectRatio) {
    final displayRect = Rect.fromLTWH(
      display.x.toDouble() * aspectRatio,
      display.y.toDouble() * aspectRatio,
      display.width.toDouble() * aspectRatio,
      display.height.toDouble() * aspectRatio,
    );

    final displayPaint = Paint()
      ..color = theme.buttonTheme.colorScheme?.background ?? Colors.white
      ..style = PaintingStyle.fill;

    final borderPaint = Paint()
      ..color = theme.buttonTheme.colorScheme?.shadow ?? Colors.red
      ..strokeWidth = 4.0
      ..style = PaintingStyle.stroke;

    canvas.drawRect(displayRect, displayPaint);
    canvas.drawRect(displayRect, borderPaint);

    final String displayLabel;
    if(display.primary) {
      displayLabel = "${display.name}\n(primary)";
    } else {
      displayLabel = display.name;
    }

    final nameSpan = TextSpan(
      text: displayLabel,
      style: TextStyle(
        color: theme.buttonTheme.colorScheme?.onBackground ?? Colors.black
      ),
    );

    final namePainter = TextPainter(
      text: nameSpan,
      textAlign: TextAlign.center,
      textDirection: TextDirection.ltr,
    );
    namePainter.layout(maxWidth: displayRect.width - 20);

    final nameOffset = displayRect.center.translate(
      -(namePainter.width / 2),
      -(namePainter.height / 2),
    );

    namePainter.paint(canvas, nameOffset);
  }

  @override
  bool shouldRepaint(_DisplayEditorPainter oldDelegate) {
    return oldDelegate.displays != displays;
  }

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
