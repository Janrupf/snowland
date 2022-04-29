use snowland_ipc::{snowland_mio, snowland_mio_misc, SnowlandIPC};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

/// Callback function type used for the API to execute synchronous callbacks on the
/// runner thread.
pub type SnowlandAPICallback =
    unsafe extern "C" fn(data: *mut std::ffi::c_void, connection: usize, message: SnowlandAPIEvent);

const MESSAGE_CHANNEL_TOKEN: snowland_mio::Token = snowland_mio::Token(1);

/// The state of a snowland api connection.
#[repr(C)]
pub enum SnowlandAPIConnectionState {
    /// The connection is connected
    Connected,

    /// The connection is disconnected
    Disconnected,
}

/// Cover type for all events that may be emitted by the API.
///
/// An event is sent every time the api user needs to be notified of a change.
#[repr(C)]
pub enum SnowlandAPIEvent {
    /// The API determined the instance id's of all alive daemons
    ///
    /// This event is always sent on the instance id 0.
    AliveConnections {
        /// The amount of id's stored in the data array
        count: usize,

        /// An array of all alive id's
        data: *const usize,
    },

    /// The state of some connection changed
    ConnectionStateChanged(SnowlandAPIConnectionState),
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
    // Bring all variables into scope, we no longer need the entire API
    let SnowlandAPI {
        receiver,
        channel_queue,
        channel_id,
        mut poll,
        data,
    } = *Box::from_raw(api);

    // Loop storage
    let mut events = snowland_mio::Events::with_capacity(1024);
    let mut connections = HashMap::new();

    loop {
        // Check for new events at each loop iteration
        poll.poll(&mut events, None).unwrap();

        for event in &events {
            if event.token() == MESSAGE_CHANNEL_TOKEN {
                // We only have one message channel, so we should have received a message on it
                assert_eq!(channel_queue.pop(), Some(channel_id));

                // Receive the message, this should never fail
                let message = receiver.recv().unwrap();

                match message {
                    SnowlandAPIMessage::ListAlive => {
                        // Currently only instance id 1 is supported, so for now we return this
                        // static data
                        let connections = [1];

                        let response = SnowlandAPIEvent::AliveConnections {
                            count: connections.len(),
                            data: connections.as_ptr(),
                        };

                        callback(data, 0, response);
                    }
                    SnowlandAPIMessage::Connect(instance) => {
                        if instance != 1 {
                            log::warn!(
                                "Attempted to connect to unsupported instance id {}",
                                instance
                            );

                            // Only instance id 1 is supported for now
                            callback(
                                data,
                                instance,
                                SnowlandAPIEvent::ConnectionStateChanged(
                                    SnowlandAPIConnectionState::Disconnected,
                                ),
                            );

                            continue;
                        }

                        // Connect the client
                        let mut ipc = match SnowlandIPC::connect_client() {
                            Err(err) => {
                                log::error!(
                                    "Failed to connect to IPC instance {}: {}",
                                    instance,
                                    err
                                );

                                // Nope, didn't work
                                callback(
                                    data,
                                    instance,
                                    SnowlandAPIEvent::ConnectionStateChanged(
                                        SnowlandAPIConnectionState::Disconnected,
                                    ),
                                );

                                continue;
                            }
                            Ok(v) => v,
                        };

                        // Register the IPC channel with our registry so we can receive events
                        ipc.register(poll.registry()).unwrap();
                        connections.insert(instance, ipc);
                    }

                    SnowlandAPIMessage::Disconnect(instance) => match connections.remove(&instance)
                    {
                        None => {
                            log::warn!("Tried to close connection to instance {}, but there was no open connection to this instance", instance);
                        }
                        Some(v) => {
                            drop(v); // TODO: Maybe add a close call?
                        }
                    },
                }
            }

            if let Entry::Occupied(mut entry) = connections.entry(1) {
                // Found an IPC instance
                let remove = {
                    let ipc = entry.get_mut();
                    if !ipc.consumes_event(event) {
                        // No need to remove the instance if it didn't even process the event
                        false
                    } else if let Err(err) = ipc.process_event(event, poll.registry()) {
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
    }
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
/// # Safety
/// This function may only be called with a valid sender pointer, the function then takes ownership
/// of the pointer.
pub unsafe extern "C" fn snowland_api_shutdown(sender: *mut SnowlandMessageSender) {
    let sender = Box::from_raw(sender);
    drop(sender);
}
