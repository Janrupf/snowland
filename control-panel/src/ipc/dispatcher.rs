use crate::ipc::InternalMessage;
use crate::util::{structure_to_value, ReserializeError};
use mio::{Events, Poll, Token, Waker};
use mio_misc::channel::Sender;
use mio_misc::queue::NotificationQueue;
use mio_misc::NotificationId;
use nativeshell::codec::{MethodCallResult, Value};
use nativeshell::shell::{EngineHandle, RunLoopSender};
use nativeshell::Context;
use serde::{Serialize, Serializer};
use snowland_ipc::protocol::ServerMessage;
use snowland_ipc::{mio, SnowlandIPC, SnowlandIPCError};
use snowland_misc::delayed::DelayedResolver;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use thiserror::Error;

const IPC_DISPATCHER_WAKER_TOKEN: Token = Token(0);

pub(super) fn ipc_dispatcher_main(
    sender: RunLoopSender,
    state: Arc<Mutex<IPCDispatcherState>>,
    sender_resolver: DelayedResolver<Result<Sender<InternalMessage>, IPCDispatcherError>>,
    engine: EngineHandle,
) {
    log::debug!("Starting ipc dispatcher...");

    // Create a poll instance, this is a precondition for even being able to talk back to the
    // platform thread.
    let poll = match mio::Poll::new() {
        Ok(v) => v,
        Err(err) => {
            sender_resolver.resolve(Err(err.into()));
            return;
        }
    };

    // We need a waker for talking as well.
    let waker = match Waker::new(poll.registry(), IPC_DISPATCHER_WAKER_TOKEN) {
        Ok(v) => Arc::new(v),
        Err(err) => {
            sender_resolver.resolve(Err(err.into()));
            return;
        }
    };

    // Create a notification queue and unique id for the talkback channel.
    let queue = Arc::new(NotificationQueue::new(waker));
    let channel_id = NotificationId::gen_next();

    // Create the channel itself, this is our main way of communication from the platform thread
    // to the ipc dispatcher.
    let (resolved, receiver) = mio_misc::channel::channel(queue, channel_id);

    // Give the platform thread a way to send to our channel.
    sender_resolver.resolve(Ok(resolved));

    log::debug!("IPC dispatcher started, entering loop...");

    // Now IPC can start...
    match ipc_dispatcher_loop(poll, receiver, &sender, &state, engine) {
        Ok(()) => {
            log::debug!("IPC dispatcher finished successfully!");
            set_state(&sender, &state, IPCDispatcherState::NotRunning, engine);
        }

        Err(err) => {
            log::error!("IPC dispatcher terminated with an error: {}", err);
            set_state(&sender, &state, IPCDispatcherState::Errored(err), engine);
        }
    }
}

fn ipc_dispatcher_loop(
    mut poll: Poll,
    receiver: Receiver<InternalMessage>,
    sender: &RunLoopSender,
    state: &Arc<Mutex<IPCDispatcherState>>,
    engine: EngineHandle,
) -> Result<(), IPCDispatcherError> {
    // Attempt to connect to the IPC or fail the operation.
    let mut ipc = match SnowlandIPC::connect_client() {
        Ok(v) => v,
        Err(SnowlandIPCError::Disconnected) => {
            log::warn!("IPC is daemon is not running, nothing to do!");
            return Ok(());
        }
        Err(err) => return Err(err.into()),
    };

    ipc.register(poll.registry())?;

    // Since we have created the channel, we can now set the state to running.
    set_state(sender, state, IPCDispatcherState::Running, engine);

    let mut events = Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            if ipc.consumes_event(event) {
                let messages = ipc.process_event(event, poll.registry())?;

                for message in messages {
                    handle_ipc_message(message, sender, engine);
                }
            } else if event.token() == IPC_DISPATCHER_WAKER_TOKEN && event.is_readable() {
                let message = match receiver.try_recv() {
                    Ok(v) => v,
                    Err(err) => {
                        log::error!("Failed to receive message after waker woke up: {}", err);
                        continue;
                    }
                };

                match message {
                    InternalMessage::SendIPC(msg) => {
                        log::trace!("Sending client message over dispatcher: {:#?}", msg);
                        ipc.evented_write(msg, poll.registry())?
                    }
                    InternalMessage::Shutdown => return Ok(()),
                }
            } else {
                log::warn!("Unexpected MIO event: {:?}", event);
            }
        }
    }
}

fn handle_ipc_message(message: ServerMessage, sender: &RunLoopSender, engine: EngineHandle) {
    match message {
        ServerMessage::UpdateConfiguration(configuration) => {
            /*
             * The configuration needs to be convert into a dart friendly representation.
             *
             * All of this dance is required because a serde_json::Value can't be sent over IPC and
             * thus has to be converted into a snowland_ipc::protocol::structure::Structure. However,
             * since the structure uses strong typing which results in artifacts and a bad object
             * structure, we have to convert it back to a dart value before serializing the entire
             * struct into a dart value.
             */

            #[derive(serde::Serialize)]
            struct InstalledModuleDart {
                pub ty: String,
                pub configuration: nativeshell::codec::Value,
            }

            #[derive(serde::Serialize)]
            struct ConfigurationDart {
                pub modules: Vec<InstalledModuleDart>,
            }

            let configuration = ConfigurationDart {
                modules: configuration
                    .modules
                    .into_iter()
                    .map(|m| InstalledModuleDart {
                        ty: m.ty,
                        configuration: structure_to_value(m.configuration),
                    })
                    .collect(),
            };

            invoke_dart_method(sender, engine, "update_configuration", configuration);
        }
        ServerMessage::UpdateDisplays(displays) => {
            send_event(sender, engine, "ipc_display_event", displays)
                .expect("Failed to serialize displays")
        }
        ServerMessage::Heartbeat => {}
    }
}

fn invoke_dart_method<V: Serialize + Send>(
    sender: &RunLoopSender,
    engine: EngineHandle,
    name: impl Into<String>,
    arg: V,
) {
    let value = match crate::util::reserialize(arg) {
        Err(err) => {
            log::error!("Failed to reserialize value: {}", err);
            return;
        }
        Ok(v) => v,
    };

    let name = name.into();

    log::trace!("Scheduling to invoke dart method {}", name);

    sender.send(move || {
        let context = match Context::current() {
            None => {
                log::error!("No context found in run loop!");
                return;
            }
            Some(v) => v,
        };

        let invoker = context
            .message_manager
            .borrow()
            .get_method_invoker(engine, "snowland_native_to_dart");

        // log::trace!("Invoking dart method {} with argument {:#?}", name, value);

        if let Err(err) = invoker.call_method(&name, value, handle_call_result) {
            log::error!("Failed to invoke dart method {}: {}", name, err);
        }
    });
}

fn handle_call_result(call_result: MethodCallResult<Value>) {
    if let Err(err) = call_result {
        log::error!("The invoked dart method failed: {}", err);
    }
}

/// Helper function to set the state on both the Rust side and signal the change to flutter.
fn set_state(
    sender: &RunLoopSender,
    state: &Arc<Mutex<IPCDispatcherState>>,
    new_state: IPCDispatcherState,
    engine: EngineHandle,
) {
    log::debug!("Changing IPC state to {:?}", new_state);

    send_event(sender, engine, "ipc_state_event", &new_state)
        .expect("Failed to serialize IPC state");

    // Set the state on the Rust side
    let mut state_lock = state.lock().unwrap();
    *state_lock = new_state;
    drop(state_lock);
}

/// Sends an event to a flutter event channel.
fn send_event<V: Serialize>(
    sender: &RunLoopSender,
    engine: EngineHandle,
    channel: impl Into<String>,
    value: V,
) -> Result<(), ReserializeError> {
    let channel = channel.into();
    let serialized_value = crate::util::reserialize(value)?;

    sender.send(move || {
        // This executes in the flutter thread, so we _should_ have a context...
        let context = match Context::current() {
            None => {
                // Should probably never happen
                log::error!("No context in thread where the flutter loop resides!");
                return;
            }
            Some(v) => v,
        };

        if let Err(err) = context
            .message_manager
            .borrow()
            .get_event_sender(engine, channel.as_ref())
            .send_event(&serialized_value)
        {
            // This should never happen neither...
            log::error!("Failed to dispatch event to flutter: {}", err);
        };
    });

    Ok(())
}

#[derive(Debug, Error)]
pub enum IPCDispatcherError {
    #[error("I/O error in IPC dispatcher: {0}")]
    IoError(#[from] std::io::Error),

    #[error("IPC failed: {0}")]
    IPCError(#[from] SnowlandIPCError),
}

impl Serialize for IPCDispatcherError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Serialize)]
pub(super) enum IPCDispatcherState {
    NotRunning,

    Running,

    Errored(IPCDispatcherError),
}

impl PartialEq for IPCDispatcherState {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Running, Self::Running)
                | (Self::NotRunning, Self::NotRunning)
                | (Self::Errored(_), Self::Errored(_))
        )
    }
}
