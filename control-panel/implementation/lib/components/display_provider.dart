import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:snowland_control_panel/data/display.dart';

const ipcDisplayChannel = EventChannel("ipc_display_event");

/// Helper widget to provide displays to the widget tree downstream.
class DisplayProvider extends StatelessWidget {
  /// Builder called when the display information is still loading.
  final WidgetBuilder loadingBuilder;

  /// Builder called when the display information is available.
  final WidgetBuilder availableBuilder;

  const DisplayProvider({
    Key? key,
    required this.loadingBuilder,
    required this.availableBuilder,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) => StreamBuilder(
        stream: ipcDisplayChannel.receiveBroadcastStream(),
        builder: (context, snapshot) {
          if (!snapshot.hasData) {
            return loadingBuilder(context);
          }

          final displays = Display.fromDataList(snapshot.data);
          return _InternalDisplayProvider(
            builder: availableBuilder,
            displays: displays,
          );
        },
      );

  /// Searches the display provider of a [context] upstream in the widget
  /// tree.
  static DisplayProviderState of(BuildContext context) {
    final state = context.findAncestorStateOfType<DisplayProviderState>();
    if (state == null) {
      throw StateError(
          "Tried to access display state outside of a display provider");
    }

    return state;
  }
}

class _InternalDisplayProvider extends StatefulWidget {
  final WidgetBuilder builder;
  final List<Display> displays;

  const _InternalDisplayProvider({
    Key? key,
    required this.builder,
    required this.displays,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => DisplayProviderState();
}

class DisplayProviderState extends State<_InternalDisplayProvider> {
  @override
  Widget build(BuildContext context) => widget.builder(context);

  List<Display> get displays => widget.displays;
}
