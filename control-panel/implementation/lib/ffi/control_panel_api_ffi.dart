import 'dart:ffi' as ffi;
import 'package:ffi/ffi.dart' as ffi;

/// Opaque message sender used to dispatch messages
class _SnowlandMessageSender extends ffi.Opaque {}

/// Opaque API representing an instance of the snowland API
class _SnowlandAPI extends ffi.Opaque {}

/// Wrapper container for API initialization
class _ExternalSnowlandAPI extends ffi.Struct {
  /// Real snowland api instance
  external ffi.Pointer<_SnowlandAPI> api;

  /// Message sender for sending messages to the API
  external ffi.Pointer<_SnowlandMessageSender> sender;
}

class _SnowlandAPIEvent extends ffi.Opaque {}

/*
 * Type declaration for
 * void (*SnowlandAPICallback)(
 *    void *data,
 *    size_t connection,
 *    struct SnowlandAPIEvent message
 * );
 */
typedef _SnowlandAPICallbackFnNative = ffi.Void Function(
    ffi.Pointer<ffi.Void>, ffi.IntPtr, _SnowlandAPIEvent);
typedef SnowlandAPICallbackFn = void Function(
    ffi.Pointer<ffi.Void>, ffi.IntPtr, _SnowlandAPIEvent);

/*
 * Type declarations for
 * ExternalSnowlandAPI snowland_api_new(void *data);
 */
typedef _SnowlandAPINewFnNative = _ExternalSnowlandAPI Function(
    ffi.Pointer<ffi.Void>);
typedef SnowlandAPINewFn = _ExternalSnowlandAPI Function(ffi.Pointer<ffi.Void>);

/*
 * Type declarations for
 * void snowland_api_run(struct SnowlandAPI *api, SnowlandAPICallback callback);
 */
typedef _SnowlandAPIRunFnNative = ffi.Void Function(ffi.Pointer<_SnowlandAPI>,
    ffi.Pointer<ffi.NativeFunction<_SnowlandAPICallbackFnNative>>);
typedef SnowlandAPIRunFn = void Function(ffi.Pointer<_SnowlandAPI>,
    ffi.Pointer<ffi.NativeFunction<_SnowlandAPICallbackFnNative>>);

/*
 * Type declarations for
 * void snowland_api_list_alive(struct SnowlandMessageSender *sender);
 */
typedef _SnowlandAPIListAliveFnNative = ffi.Void Function(
    ffi.Pointer<_SnowlandMessageSender>);
typedef SnowlandAPIListAliveFn = void Function(
    ffi.Pointer<_SnowlandMessageSender>);

/*
 * Type declarations for
 * void snowland_api_connect(
 *    struct SnowlandMessageSender *sender,
 *    size_t instance
 * );
 */
typedef _SnowlandAPIConnectFnNative = ffi.Void Function(
    ffi.Pointer<_SnowlandMessageSender>, ffi.IntPtr);
typedef SnowlandAPIConnectFn = void Function(
    ffi.Pointer<_SnowlandMessageSender>, int);

/*
 * Type declarations for
 * void snowland_api_disconnect(
 *    struct SnowlandMessageSender *sender,
 *    size_t instance
 * );
 */
typedef _SnowlandAPIDisconnectFnNative = ffi.Void Function(
    ffi.Pointer<_SnowlandMessageSender>, ffi.IntPtr);
typedef SnowlandAPIDisconnectFn = void Function(
    ffi.Pointer<_SnowlandMessageSender>, int);

/*
 * Type declarations for
 * void snowland_api_shutdown(struct SnowlandMessageSender *sender);
 */
typedef _SnowlandAPIShutdownFnNative = ffi.Void Function(
    ffi.Pointer<_SnowlandMessageSender>);
typedef SnowlandAPIShutdownFn = void Function(
    ffi.Pointer<_SnowlandMessageSender>);

/*
 * Type declarations for
 * void snowland_api_init_logging();
 */
typedef _SnowlandAPIInitLoggingFnNative = ffi.Void Function();
typedef SnowlandAPIInitLoggingFn = void Function();

/*
 * Type declarations for
 * void snowland_api_log(
 *    const char *component,
 *    const char *level,
 *    const char *message
 * );
 */
typedef _SnowlandAPILogFnNative = ffi.Void Function(
    ffi.Pointer<ffi.Utf8>, ffi.Pointer<ffi.Utf8>, ffi.Pointer<ffi.Utf8>);
typedef SnowlandAPILogFn = void Function(
    ffi.Pointer<ffi.Utf8>, ffi.Pointer<ffi.Utf8>, ffi.Pointer<ffi.Utf8>);

class ControlPanelAPIFFI {
  factory ControlPanelAPIFFI() {
    final library = ffi.DynamicLibrary.open(
        "/projects/public/snowland/target/debug/libsnowland_api.so");

    final createNew =
        library.lookupFunction<_SnowlandAPINewFnNative, SnowlandAPINewFn>(
            "snowland_api_new");

    final run =
        library.lookupFunction<_SnowlandAPIRunFnNative, SnowlandAPIRunFn>(
            "snowland_api_run");

    final listAlive = library.lookupFunction<_SnowlandAPIListAliveFnNative,
        SnowlandAPIListAliveFn>("snowland_api_list_alive");

    final connect = library.lookupFunction<_SnowlandAPIConnectFnNative,
        SnowlandAPIConnectFn>("snowland_api_connect");

    final disconnect = library.lookupFunction<_SnowlandAPIDisconnectFnNative,
        SnowlandAPIDisconnectFn>("snowland_api_disconnect");

    final shutdown = library.lookupFunction<_SnowlandAPIShutdownFnNative,
        SnowlandAPIShutdownFn>("snowland_api_shutdown");

    final initLogging = library.lookupFunction<_SnowlandAPIInitLoggingFnNative,
        SnowlandAPIInitLoggingFn>("snowland_api_init_logging");

    final log =
        library.lookupFunction<_SnowlandAPILogFnNative, SnowlandAPILogFn>(
            "snowland_api_log");

    return ControlPanelAPIFFI._(
      createNew: createNew,
      run: run,
      listAlive: listAlive,
      connect: connect,
      disconnect: disconnect,
      shutdown: shutdown,
      initLogging: initLogging,
      log: log,
    );
  }

  const ControlPanelAPIFFI._({
    required this.createNew,
    required this.run,
    required this.listAlive,
    required this.connect,
    required this.disconnect,
    required this.shutdown,
    required this.initLogging,
    required this.log,
  });

  final SnowlandAPINewFn createNew;
  final SnowlandAPIRunFn run;
  final SnowlandAPIListAliveFn listAlive;
  final SnowlandAPIConnectFn connect;
  final SnowlandAPIDisconnectFn disconnect;
  final SnowlandAPIShutdownFn shutdown;
  final SnowlandAPIInitLoggingFn initLogging;
  final SnowlandAPILogFn log;
}
