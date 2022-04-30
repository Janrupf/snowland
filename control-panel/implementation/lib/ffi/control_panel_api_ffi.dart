import 'dart:collection';
import 'dart:ffi' as ffi;
import 'package:ffi/ffi.dart' as ffi;

/// Opaque message sender used to dispatch messages
class SnowlandMessageSender extends ffi.Opaque {}

/// Opaque API representing an instance of the snowland API
class SnowlandAPI extends ffi.Opaque {}

/// Wrapper container for API initialization
class ExternalSnowlandAPI extends ffi.Struct {
  /// Real snowland api instance
  external ffi.Pointer<SnowlandAPI> api;

  /// Message sender for sending messages to the API
  external ffi.Pointer<SnowlandMessageSender> sender;
}

/// Opaque container for events returned by the API
class SnowlandAPIEventsNative extends ffi.Opaque {}

/// Native structure received in the callback function
class SnowlandAPIEventNative extends ffi.Struct {
  /// Tag identifying the union content
  @ffi.IntPtr()
  external int _tag;

  /// The underlying union data
  external _SnowlandAPIEventData _data;

  /// Attempts to decode the union content into a dart friendly representation
  ///
  /// When decoding fails, this throws an [UnimplementedError].
  SnowlandAPIEvent decode() {
    switch (_tag) {
      case 0:
        // This does not contain any data, it's just a notification
        return SnowlandAPIEventDispatchRuntimeEvents._();

      case 1:
        {
          // Re-assemble the native array
          final data = _data._aliveConnections;
          final alive = List.generate(data._count, (i) => data._data[i]);

          return SnowlandAPIEventAliveConnections._(alive);
        }

      case 2:
        {
          // The data is an int describing the new connection state
          final data = _data._connectionState;

          switch (data) {
            case 0:
              return SnowlandAPIEventConnectionState.connected;

            case 1:
              return SnowlandAPIEventConnectionState.disconnected;

            default:
              throw UnimplementedError(
                  "Got invalid SnowlandAPI connection state $data");
          }
        }

      case 3:
        return SnowlandAPIEventShutdown._();

      default:
        throw UnimplementedError("Got invalid SnowlandAPI event tag $_tag");
    }
  }
}

/// Inner data union of a [SnowlandAPIEventNative]
class _SnowlandAPIEventData extends ffi.Union {
  /// Data received for alive connection events
  external SnowlandAPIEventAliveConnectionsNative _aliveConnections;

  /// Data received for connection state changes
  @ffi.IntPtr()
  external int _connectionState;
}

/// Base class for the dart representation of all API events
abstract class SnowlandAPIEvent {}

/// Something interrupted the snowland API while waiting for events
/// and now it called back to allow processing of whatever interrupt
/// was received.
///
/// In the case of the dart runtime it seems to be enough to "not handle"
/// the interrupt, because running dart code alone will be enough to drain
/// the event loop.
class SnowlandAPIEventDispatchRuntimeEvents implements SnowlandAPIEvent {
  SnowlandAPIEventDispatchRuntimeEvents._();
}

/// Native event received when a listing of alive connections was
/// requested
class SnowlandAPIEventAliveConnectionsNative extends ffi.Struct {
  /// The count of elements in the array pointed to by [_data]
  @ffi.IntPtr()
  external int _count;

  /// Pointer to the first element of an array of the length [_count]
  external ffi.Pointer<ffi.IntPtr> _data;
}

/// Dart friendly representation of the alive connections event
class SnowlandAPIEventAliveConnections implements SnowlandAPIEvent {
  final List<int> _alive;

  SnowlandAPIEventAliveConnections._(this._alive);

  /// Retrieves a list of all alive snowland connections
  UnmodifiableListView<int> get alive => UnmodifiableListView(_alive);
}

/// Dart friendly representation of the connection state change event
enum SnowlandAPIEventConnectionState implements SnowlandAPIEvent {
  /// The connection connected
  connected,

  /// The connection disconnected and is inoperable
  disconnected
}

/// Dart friendly representation of the shutdown event
class SnowlandAPIEventShutdown implements SnowlandAPIEvent {
  SnowlandAPIEventShutdown._();
}

/*
 * Type declaration for
 * void (*SnowlandAPICallback)(
 *    void *data,
 *    size_t connection,
 *    struct SnowlandAPIEvent message
 * );
 */
typedef SnowlandAPICallbackFnNative = ffi.Void Function(
    ffi.Pointer<ffi.Void>, ffi.IntPtr, SnowlandAPIEventNative);

/*
 * Type declarations for
 * ExternalSnowlandAPI snowland_api_new(void *data);
 */
typedef _SnowlandAPINewFnNative = ExternalSnowlandAPI Function(
    ffi.Pointer<ffi.Void>);
typedef SnowlandAPINewFn = ExternalSnowlandAPI Function(ffi.Pointer<ffi.Void>);

/*
 * Type declarations for
 * void snowland_api_run(struct SnowlandAPI *api, SnowlandAPICallback callback);
 */
typedef _SnowlandAPIRunFnNative = ffi.Void Function(ffi.Pointer<SnowlandAPI>,
    ffi.Pointer<ffi.NativeFunction<SnowlandAPICallbackFnNative>>);
typedef SnowlandAPIRunFn = void Function(ffi.Pointer<SnowlandAPI>,
    ffi.Pointer<ffi.NativeFunction<SnowlandAPICallbackFnNative>>);

/*
 * Type declarations for
 * struct SnowlandAPIEvents *snowland_api_poll(struct SnowlandAPI *api);
 */
typedef _SnowlandAPIPollFnNative = ffi.Pointer<SnowlandAPIEventsNative>
    Function(ffi.Pointer<SnowlandAPI>);
typedef SnowlandAPIPollFn = ffi.Pointer<SnowlandAPIEventsNative> Function(
    ffi.Pointer<SnowlandAPI>);

/*
 * Type declarations for
 * size_t snowland_api_event_count(struct SnowlandAPIEvents *events);
 */
typedef _SnowlandAPIEventCountFnNative = ffi.IntPtr Function(
    ffi.Pointer<SnowlandAPIEventsNative>);
typedef SnowlandAPIEventCountFn = int Function(
    ffi.Pointer<SnowlandAPIEventsNative>);

/*
 * Type declarations for
 * size_t snowland_api_get_event_connection_id(
 *    struct SnowlandAPIEvents *events,
 *    size_t index
 * );
 */
typedef _SnowlandAPIGetEventConnectionIdFnNative = ffi.IntPtr Function(
    ffi.Pointer<SnowlandAPIEventsNative>, ffi.IntPtr);
typedef SnowlandAPIGetEventConnectionIdFn = int Function(
    ffi.Pointer<SnowlandAPIEventsNative>, int);

/*
 * Type declarations for
 * const struct SnowlandAPIEvent *snowland_api_get_event_data(
 *    struct SnowlandAPIEvents *events,
 *    size_t index
 * );
 */
typedef _SnowlandAPIGetEventDataFnNative = ffi.Pointer<SnowlandAPIEventNative>
    Function(ffi.Pointer<SnowlandAPIEventsNative>, ffi.IntPtr);
typedef SnowlandAPIGetEventDataFn = ffi.Pointer<SnowlandAPIEventNative>
    Function(ffi.Pointer<SnowlandAPIEventsNative>, int);

/*
 * Type declarations for
 * void snowland_api_free_events(struct SnowlandAPIEvents *events);
 */
typedef _SnowlandAPIFreeEventsFnNative = ffi.Void Function(
    ffi.Pointer<SnowlandAPIEventsNative>);
typedef SnowlandAPIFreeEventsFn = void Function(
    ffi.Pointer<SnowlandAPIEventsNative>);

/*
 * Type declarations for
 * void snowland_api_list_alive(struct SnowlandMessageSender *sender);
 */
typedef _SnowlandAPIListAliveFnNative = ffi.Void Function(
    ffi.Pointer<SnowlandMessageSender>);
typedef SnowlandAPIListAliveFn = void Function(
    ffi.Pointer<SnowlandMessageSender>);

/*
 * Type declarations for
 * void snowland_api_connect(
 *    struct SnowlandMessageSender *sender,
 *    size_t instance
 * );
 */
typedef _SnowlandAPIConnectFnNative = ffi.Void Function(
    ffi.Pointer<SnowlandMessageSender>, ffi.IntPtr);
typedef SnowlandAPIConnectFn = void Function(
    ffi.Pointer<SnowlandMessageSender>, int);

/*
 * Type declarations for
 * void snowland_api_disconnect(
 *    struct SnowlandMessageSender *sender,
 *    size_t instance
 * );
 */
typedef _SnowlandAPIDisconnectFnNative = ffi.Void Function(
    ffi.Pointer<SnowlandMessageSender>, ffi.IntPtr);
typedef SnowlandAPIDisconnectFn = void Function(
    ffi.Pointer<SnowlandMessageSender>, int);

/*
 * Type declarations for
 * void snowland_api_shutdown(struct SnowlandMessageSender *sender);
 */
typedef _SnowlandAPIShutdownFnNative = ffi.Void Function(
    ffi.Pointer<SnowlandMessageSender>);
typedef SnowlandAPIShutdownFn = void Function(
    ffi.Pointer<SnowlandMessageSender>);

/*
 * Type declarations for
 * void snowland_api_free(struct SnowlandAPI *api);
 */
typedef _SnowlandAPIFreeFnNative = ffi.Void Function(ffi.Pointer<SnowlandAPI>);
typedef SnowlandAPIFreeFn = void Function(ffi.Pointer<SnowlandAPI>);

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

    final poll =
        library.lookupFunction<_SnowlandAPIPollFnNative, SnowlandAPIPollFn>(
            "snowland_api_poll");

    final eventCount = library.lookupFunction<_SnowlandAPIEventCountFnNative,
        SnowlandAPIEventCountFn>("snowland_api_event_count");

    final getEventConnectionId = library.lookupFunction<
            _SnowlandAPIGetEventConnectionIdFnNative,
            SnowlandAPIGetEventConnectionIdFn>(
        "snowland_api_get_event_connection_id");

    final getEventData = library.lookupFunction<
        _SnowlandAPIGetEventDataFnNative,
        SnowlandAPIGetEventDataFn>("snowland_api_get_event_data");

    final freeEvents = library.lookupFunction<_SnowlandAPIFreeEventsFnNative,
        SnowlandAPIFreeEventsFn>("snowland_api_free_events");

    final listAlive = library.lookupFunction<_SnowlandAPIListAliveFnNative,
        SnowlandAPIListAliveFn>("snowland_api_list_alive");

    final connect = library.lookupFunction<_SnowlandAPIConnectFnNative,
        SnowlandAPIConnectFn>("snowland_api_connect");

    final disconnect = library.lookupFunction<_SnowlandAPIDisconnectFnNative,
        SnowlandAPIDisconnectFn>("snowland_api_disconnect");

    final shutdown = library.lookupFunction<_SnowlandAPIShutdownFnNative,
        SnowlandAPIShutdownFn>("snowland_api_shutdown");

    final free =
        library.lookupFunction<_SnowlandAPIFreeFnNative, SnowlandAPIFreeFn>(
            "snowland_api_free");

    final initLogging = library.lookupFunction<_SnowlandAPIInitLoggingFnNative,
        SnowlandAPIInitLoggingFn>("snowland_api_init_logging");

    final log =
        library.lookupFunction<_SnowlandAPILogFnNative, SnowlandAPILogFn>(
            "snowland_api_log");

    return ControlPanelAPIFFI._(
      createNew: createNew,
      run: run,
      poll: poll,
      eventCount: eventCount,
      getEventConnectionId: getEventConnectionId,
      getEventData: getEventData,
      freeEvents: freeEvents,
      listAlive: listAlive,
      connect: connect,
      disconnect: disconnect,
      shutdown: shutdown,
      free: free,
      initLogging: initLogging,
      log: log,
    );
  }

  const ControlPanelAPIFFI._({
    required this.createNew,
    required this.run,
    required this.poll,
    required this.eventCount,
    required this.getEventConnectionId,
    required this.getEventData,
    required this.freeEvents,
    required this.listAlive,
    required this.connect,
    required this.disconnect,
    required this.shutdown,
    required this.free,
    required this.initLogging,
    required this.log,
  });

  final SnowlandAPINewFn createNew;
  final SnowlandAPIRunFn run;
  final SnowlandAPIPollFn poll;
  final SnowlandAPIEventCountFn eventCount;
  final SnowlandAPIGetEventConnectionIdFn getEventConnectionId;
  final SnowlandAPIGetEventDataFn getEventData;
  final SnowlandAPIFreeEventsFn freeEvents;
  final SnowlandAPIListAliveFn listAlive;
  final SnowlandAPIConnectFn connect;
  final SnowlandAPIDisconnectFn disconnect;
  final SnowlandAPIShutdownFn shutdown;
  final SnowlandAPIFreeFn free;
  final SnowlandAPIInitLoggingFn initLogging;
  final SnowlandAPILogFn log;
}
