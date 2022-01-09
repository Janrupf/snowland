use crate::ipc::InternalMessage;
use mio::{Events, Poll, Token, Waker};
use mio_misc::channel::Sender;
use mio_misc::queue::NotificationQueue;
use mio_misc::NotificationId;
use nativeshell::shell::{EngineHandle, RunLoopSender};
use nativeshell::Context;
use serde::{Serialize, Serializer};
use snowland_ipc::protocol::{ClientMessage, ServerMessage};
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
                    handle_ipc_message(message, sender);
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
                    InternalMessage::SendIPC(msg) => ipc.evented_write(msg, poll.registry())?,
                    InternalMessage::Shutdown => return Ok(()),
                }
            } else {
                log::warn!("Unexpected MIO event: {:?}", event);
            }
        }
    }
}

fn handle_ipc_message(message: ServerMessage, sender: &RunLoopSender) {}

/// Helper function to set the state on both the Rust side and signal the change to flutter.
fn set_state(
    sender: &RunLoopSender,
    state: &Arc<Mutex<IPCDispatcherState>>,
    new_state: IPCDispatcherState,
    engine: EngineHandle,
) {
    log::debug!("Changing IPC state to {:?}", new_state);

    // Convert the state into a value which can be sent to flutter
    let serialized_state =
        crate::util::reserialize(&new_state).expect("Failed to serialize IPC state");

    // Set the state on the Rust side
    let mut state_lock = state.lock().unwrap();
    *state_lock = new_state;
    drop(state_lock);

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
            .get_event_sender(engine, "ipc_state_event")
            .send_event(&serialized_state)
        {
            // This should never happen neither...
            log::error!("Failed to dispatch ipc state change to flutter: {}", err);
        };
    });
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
