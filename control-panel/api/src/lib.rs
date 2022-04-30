use snowland_ipc::protocol::{ClientMessage, ServerMessage};
use snowland_ipc::{snowland_mio, snowland_mio_misc, SnowlandIPC};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::CStr;
use std::mem::ManuallyDrop;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

/// Callback function type used for the API to execute synchronous callbacks on the
/// runner thread.
pub type SnowlandAPICallback =
    unsafe extern "C" fn(data: *mut std::ffi::c_void, connection: usize, message: SnowlandAPIEvent);

const MESSAGE_CHANNEL_TOKEN: snowland_mio::Token = snowland_mio::Token(1);

/// The state of a snowland api connection.
#[repr(usize)]
pub enum SnowlandAPIConnectionState {
    /// The connection is connected
    Connected,

    /// The connection is disconnected
    Disconnected,
}

/// Cover type for all events that may be emitted by the API.
///
/// An event is sent every time the api user needs to be notified of a change.
#[repr(C, usize)]
pub enum SnowlandAPIEvent {
    /// The API finished a poll and now gives the calling runtime some time to process events
    DispatchRuntimeEvent,

    /// The API determined the instance id's of all alive daemons
    ///
    /// This event is always sent on the instance id 0.
    AliveInstances {
        /// The amount of id's stored in the data array
        count: usize,

        /// An array of all alive id's
        data: *const usize,
    },

    /// The state of some connection changed
    ConnectionStateChanged(SnowlandAPIConnectionState),

    /// Shutdown has been requested
    Shutdown,
}

/// Type wrapper for event structure returned by the polling api
pub struct SnowlandAPIEvents {
    inner: Vec<(usize, SnowlandAPIEvent)>,
}

impl From<Vec<(usize, SnowlandAPIEvent)>> for SnowlandAPIEvents {
    fn from(events: Vec<(usize, SnowlandAPIEvent)>) -> Self {
        Self { inner: events }
    }
}

impl From<SnowlandAPIEvents> for Vec<(usize, SnowlandAPIEvent)> {
    fn from(events: SnowlandAPIEvents) -> Self {
        events.inner
    }
}

/// Wrapper type for the initial creation of the API.
///
/// This type contains everything necessary to talk to the API.
#[repr(C)]
pub struct ExternalSnowlandAPI {
    /// The API instance itself
    ///
    /// This should be dispatched on an external thread into a run loop.
    pub api: *mut SnowlandAPI,

    /// The sender for sending messages to the API after it has been given to the run loop
    pub sender: *mut SnowlandMessageSender,
}

enum SnowlandAPIMessage {
    ListAlive,
    Connect(usize),
    Disconnect(usize),
    Shutdown,
}

/// Message sender to be used to dispatch message to the API.
pub struct SnowlandMessageSender {
    inner: snowland_mio_misc::channel::Sender<SnowlandAPIMessage>,
}

pub struct SnowlandAPI {
    receiver: Receiver<SnowlandAPIMessage>,
    channel_queue: Arc<snowland_mio_misc::queue::NotificationQueue>,
    channel_id: snowland_mio_misc::NotificationId,
    poll: snowland_mio::Poll,
    data: *mut std::ffi::c_void,
    events: snowland_mio::Events,
    connections: HashMap<usize, SnowlandIPC<ClientMessage, ServerMessage>>,
    alive: Vec<usize>,
}

/// Creates a new snowland api instance and returns a message sender and the created
/// instance.
#[no_mangle]
pub extern "C" fn snowland_api_new(data: *mut std::ffi::c_void) -> ExternalSnowlandAPI {
    // TODO: Handle errors
    let poll = snowland_mio::Poll::new().expect("failed to create poll");
    let channel_waker = Arc::new(
        snowland_mio::Waker::new(poll.registry(), MESSAGE_CHANNEL_TOKEN)
            .expect("failed to create waker"),
    );
    let channel_queue = Arc::new(snowland_mio_misc::queue::NotificationQueue::new(
        channel_waker,
    ));

    let channel_id = snowland_mio_misc::NotificationId::gen_next();

    let (sender, receiver) = snowland_mio_misc::channel::channel(channel_queue.clone(), channel_id);

    let api = Box::new(SnowlandAPI {
        receiver,
        channel_queue,
        channel_id,
        poll,
        data,
        events: snowland_mio::Events::with_capacity(1024),
        connections: HashMap::new(),
        alive: Vec::new(),
    });

    let sender = Box::new(SnowlandMessageSender { inner: sender });

    ExternalSnowlandAPI {
        sender: Box::into_raw(sender),
        api: Box::into_raw(api),
    }
}

/// Starts the run loop for the api instance.
///
/// # Safety
/// The api and callback pointer must be valid,
/// ownership of the api is transferred to this function.
#[no_mangle]
pub unsafe extern "C" fn snowland_api_run(api: *mut SnowlandAPI, callback: SnowlandAPICallback) {
    loop {
        let events = snowland_api_poll(api);
        if events.is_null() {
            continue;
        }

        let events = Box::from_raw(events);

        for (instance, event) in events.inner {
            callback((*api).data, instance, event);
        }
    }
}

/// Polls for events a single time
///
/// # Safety
/// This function may only be called with a valid api pointer.
#[no_mangle]
pub unsafe extern "C" fn snowland_api_poll(api: *mut SnowlandAPI) -> *mut SnowlandAPIEvents {
    let mut api = ManuallyDrop::new(Box::from_raw(api));
    let api = api.as_mut(); // We need to aid the borrow checker a bit

    api.alive.clear();

    // Check for new events at each loop iteration
    if let Err(err) = api.poll.poll(&mut api.events, None) {
        if err.kind() == std::io::ErrorKind::Interrupted {
            return std::ptr::null_mut();
        }
    }

    log::trace!(
        "Polling succeeded, processing {} events",
        api.events.iter().count()
    );

    let mut api_events = Vec::new();
    let mut shutdown = false;

    for event in &api.events {
        if event.token() == MESSAGE_CHANNEL_TOKEN {
            // We only have one message channel, so we should have received a message on it
            assert_eq!(api.channel_queue.pop(), Some(api.channel_id));

            // Receive the message, this should never fail
            let message = api.receiver.recv().unwrap();

            match message {
                SnowlandAPIMessage::ListAlive => {
                    api.alive = SnowlandIPC::list_alive_instances();

                    let response = SnowlandAPIEvent::AliveInstances {
                        count: api.alive.len(),
                        data: api.alive.as_ptr(),
                    };

                    api_events.push((0, response));
                }
                SnowlandAPIMessage::Connect(instance) => {
                    // Connect the client
                    let mut ipc = match SnowlandIPC::connect_client(instance) {
                        Err(err) => {
                            log::error!("Failed to connect to IPC instance {}: {}", instance, err);
                            api_events.push((
                                instance,
                                SnowlandAPIEvent::ConnectionStateChanged(
                                    SnowlandAPIConnectionState::Disconnected,
                                ),
                            ));

                            continue;
                        }
                        Ok(v) => v,
                    };

                    // Register the IPC channel with our registry so we can receive events
                    ipc.register(api.poll.registry()).unwrap();
                    api.connections.insert(instance, ipc);
                }

                SnowlandAPIMessage::Disconnect(instance) => match api.connections.remove(&instance)
                {
                    None => {
                        log::warn!("Tried to close connection to instance {}, but there was no open connection to this instance", instance);
                    }
                    Some(v) => {
                        drop(v); // TODO: Maybe add a close call?
                    }
                },

                SnowlandAPIMessage::Shutdown => shutdown = true,
            }
        }

        if let Entry::Occupied(mut entry) = api.connections.entry(1) {
            // Found an IPC instance
            let remove = {
                let ipc = entry.get_mut();
                if !ipc.consumes_event(event) {
                    // No need to remove the instance if it didn't even process the event
                    false
                } else if let Err(err) = ipc.process_event(event, api.poll.registry()) {
                    // The instance failed to process the event
                    log::error!("IPC instance {} failed to process event: {}", 1, err);
                    true
                } else {
                    // Event has been processed successfully
                    false
                }
            };

            if remove {
                // Clear the entry after an error
                entry.remove();
            }
        }
    }

    if shutdown {
        api_events.push((0, SnowlandAPIEvent::Shutdown));
    }

    let events = SnowlandAPIEvents { inner: api_events };

    Box::into_raw(Box::new(events))
}

/// Determines the amount of events stored in the structure
///
/// # Safety
/// This function may only be called with a null or valid events pointer.
#[no_mangle]
pub unsafe extern "C" fn snowland_api_event_count(events: *mut SnowlandAPIEvents) -> usize {
    if events.is_null() {
        return 0;
    }

    (*events).inner.len()
}

/// Retrieves the connection id for an event at a specific index in the event list
///
/// # Safety
/// This function may only be called with a valid events pointer and index.
#[no_mangle]
pub unsafe extern "C" fn snowland_api_get_event_connection_id(
    events: *mut SnowlandAPIEvents,
    index: usize,
) -> usize {
    (*events).inner[index].0
}

/// Retrieves the event data for an event at a specific index in the event list
///
/// # Safety
/// This function may only be called with a valid events pointer and index.
#[no_mangle]
pub unsafe extern "C" fn snowland_api_get_event_data(
    events: *mut SnowlandAPIEvents,
    index: usize,
) -> *const SnowlandAPIEvent {
    &(*events).inner[index].1
}

/// Free's the event data.
///
/// # Safety
/// This function may only be called with a valid events pointer and then takes ownership.
#[no_mangle]
pub unsafe extern "C" fn snowland_api_free_events(events: *mut SnowlandAPIEvents) {
    drop(Box::from_raw(events));
}

/// Requests a list of all alive snowland daemons.
///
/// # Safety
/// This function may only be called with a valid sender pointer.
#[no_mangle]
pub unsafe extern "C" fn snowland_api_list_alive(sender: *mut SnowlandMessageSender) {
    let sender = &(*sender).inner;
    sender.send(SnowlandAPIMessage::ListAlive).unwrap();
}

/// Initiates a connection with the specified instance id.
///
/// # Safety
/// This function may only be called with a valid sender pointer.
#[no_mangle]
pub unsafe extern "C" fn snowland_api_connect(sender: *mut SnowlandMessageSender, instance: usize) {
    let sender = &(*sender).inner;
    sender.send(SnowlandAPIMessage::Connect(instance)).unwrap();
}

/// Closes the connection with the specified instance id.
///
/// # Safety
/// This function may only be called with a valid sender pointer.
#[no_mangle]
pub unsafe extern "C" fn snowland_api_disconnect(
    sender: *mut SnowlandMessageSender,
    instance: usize,
) {
    let sender = &(*sender).inner;
    sender
        .send(SnowlandAPIMessage::Disconnect(instance))
        .unwrap();
}

/// Shuts down the entire api
///
/// The api handler then should call [`snowland_api_free`] on the api pointer.
///
/// # Safety
/// This function may only be called with a valid sender pointer, the function then takes ownership
/// of the pointer.
#[no_mangle]
pub unsafe extern "C" fn snowland_api_shutdown(sender: *mut SnowlandMessageSender) {
    let sender = Box::from_raw(sender);
    sender.inner.send(SnowlandAPIMessage::Shutdown).unwrap();
    drop(sender);
}

/// Free's the api
///
/// Should always be called after the sender has called [`snowland_api_shutdown`].
///
/// # Safety
/// This function may only be called with a valid api pointer, the function then takes ownership
/// of the pointer.
#[no_mangle]
pub unsafe extern "C" fn snowland_api_free(api: *mut SnowlandAPI) {
    drop(Box::from_raw(api));
}

/// Initializes logging
#[no_mangle]
pub extern "C" fn snowland_api_init_logging() {
    pretty_env_logger::init_timed();
}

/// Logs a message using the snowland logger.
///
/// # Safety
/// This function may only be called with all pointers being valid UTF-8 c strings.
#[no_mangle]
pub unsafe extern "C" fn snowland_api_log(
    component: *const std::os::raw::c_char,
    level: *const std::os::raw::c_char,
    message: *const std::os::raw::c_char,
) {
    let component = CStr::from_ptr(component).to_string_lossy();
    let level = CStr::from_ptr(level).to_string_lossy();
    let level = match level.as_ref() {
        "trace" => log::Level::Trace,
        "debug" => log::Level::Debug,
        "info" => log::Level::Info,
        "warn" => log::Level::Warn,
        "error" => log::Level::Error,
        _ => log::Level::Error,
    };

    let message = CStr::from_ptr(message).to_string_lossy();
    let target = format!("[[dart]]::{}", component);

    if log::log_enabled!(target: &target, level) {
        log::log!(target: &target, level, "{}", message);
    }
}
