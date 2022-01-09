use crate::ipc::InternalMessage;
use mio::{Events, Poll, Token, Waker};
use mio_misc::channel::Sender;
use mio_misc::queue::NotificationQueue;
use mio_misc::NotificationId;
use nativeshell::shell::RunLoopSender;
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

    // Since we have created the channel, we can now set the state to running.
    let mut state_lock = state.lock().unwrap();
    *state_lock = IPCDispatcherState::Running;
    drop(state_lock);

    // Give the platform thread a way to send to our channel.
    sender_resolver.resolve(Ok(resolved));

    log::debug!("IPC dispatcher started, entering loop...");

    // Now IPC can start...
    match ipc_dispatcher_loop(poll, receiver, sender) {
        Ok(()) => {
            log::debug!("IPC dispatcher finished successfully!");

            let mut state_lock = state.lock().unwrap();
            *state_lock = IPCDispatcherState::NotRunning;
            drop(state_lock);
        }

        Err(err) => {
            log::error!("IPC dispatcher terminated with an error: {}", err);

            let mut state_lock = state.lock().unwrap();
            *state_lock = IPCDispatcherState::Errored(err);
            drop(state_lock);
        }
    }
}

fn ipc_dispatcher_loop(
    mut poll: Poll,
    receiver: Receiver<InternalMessage>,
    sender: RunLoopSender,
) -> Result<(), IPCDispatcherError> {
    // Attempt to connect to the IPC or fail the operation.
    let mut ipc = SnowlandIPC::connect_client()?;
    ipc.register(poll.registry())?;

    let mut events = Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            if ipc.consumes_event(event) {
                let messages = ipc.process_event(event, poll.registry())?;

                for message in messages {
                    handle_ipc_message(message, &sender);
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

#[derive(Debug, Error)]
pub enum IPCDispatcherError {
    #[error("I/O error in IPC dispatcher: {0}")]
    IoError(#[from] std::io::Error),

    #[error("IPC failed: {0}")]
    IPCError(#[from] SnowlandIPCError),
}

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
