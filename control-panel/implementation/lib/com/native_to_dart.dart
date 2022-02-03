import 'dart:collection';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

const MethodChannel _nativeToDartChannel =
    MethodChannel("snowland_native_to_dart");

/// Signature of functions handling calls from the native side..
typedef NativeCallHandler = Future<dynamic> Function(dynamic args);

/// Helper to receive method calls from the native side.
class NativeToDartCommunicator {
  final Map<String, NativeCallHandler> _installedHandlers;

  NativeToDartCommunicator._() : _installedHandlers = HashMap() {
    _nativeToDartChannel.setMethodCallHandler((call) {
      final handler = _installedHandlers[call.method];

      if (handler != null) {
        return handler(call.arguments);
      } else {
        throw MissingPluginException(
            "No message handler installed for call ${call.method}");
      }
    });
  }

  static final NativeToDartCommunicator instance = NativeToDartCommunicator._();

  /// Installs a method handler.
  void installHandler(String method, NativeCallHandler handler) {
    _installedHandlers[method] = handler;
  }

  /// Uninstalls a method handler.
  void uninstallHandler(String method) {
    _installedHandlers.remove(method);
  }
}

/// Widget which associates a native to dart method with itself.
///
/// When the widget is inserted into the tree the first time, the method handler
/// is installed. Upon final removal of the widget the handler is uninstalled.
class NativeCallWidget extends StatefulWidget {
  final String methodName;
  final NativeCallHandler handler;
  final Widget child;

  const NativeCallWidget(
      {Key? key,
      required this.methodName,
      required this.handler,
      required this.child})
      : super(key: key);

  @override
  State<StatefulWidget> createState() => _NativeCallWidgetState();
}

class _NativeCallWidgetState extends State<NativeCallWidget> {
  @override
  void initState() {
    super.initState();
    NativeToDartCommunicator.instance
        .installHandler(widget.methodName, widget.handler);
  }

  @override
  void dispose() {
    NativeToDartCommunicator.instance.uninstallHandler(widget.methodName);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.child;
}

/// Function which is called when a [NativeCallReceiver] updates in response
/// to a method being called.
typedef NativeCallReceiverBuilder = Widget Function(
    BuildContext context, dynamic data);

/// A widget which listens to native method calls and then rebuilds each time
/// the native method is called.
class NativeCallReceiver extends StatefulWidget {
  final String methodName;
  final NativeCallReceiverBuilder builder;

  const NativeCallReceiver({
    Key? key,
    required this.methodName,
    required this.builder,
  }) : super(key: key);

  @override
  State<StatefulWidget> createState() => _NativeCallReceiverState();
}

class _NativeCallReceiverState extends State<NativeCallReceiver> {
  dynamic _currentData;

  @override
  Widget build(BuildContext context) => NativeCallWidget(
        methodName: widget.methodName,
        handler: _updateData,
        child: widget.builder(context, _currentData),
      );

  Future _updateData(dynamic newData) {
    if (mounted) {
      setState(() {
        _currentData = newData;
      });
    }

    return Future.value();
  }
}
