mod dispatcher;

pub use crate::ipc::dispatcher::IPCDispatcherError;
use crate::ipc::dispatcher::{ipc_dispatcher_main, IPCDispatcherState};
use nativeshell::shell::RunLoopSender;
use snowland_ipc::protocol::ClientMessage;
use snowland_misc::delayed::Delayed;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub struct IPCHandle {
    sender: RunLoopSender,
    state: Arc<Mutex<IPCDispatcherState>>,
    inner: Option<InnerIPCHandle>,
}

impl IPCHandle {
    pub fn new(sender: RunLoopSender) -> Self {
        let state = Arc::new(Mutex::new(IPCDispatcherState::NotRunning));

        Self {
            sender,
            state,
            inner: None,
        }
    }

    /// Starts connecting the IPC if it is not connected already.
    pub fn start_connecting(&mut self) -> Result<(), IPCDispatcherError> {
        if self.is_running() {
            return Ok(());
        }

        // Clone data that needs to be passed to the new thread
        let run_loop_sender = self.sender.clone();
        let state = self.state.clone();

        let (sender, sender_resolver) = Delayed::new();

        let ipc_thread = std::thread::spawn(move || {
            ipc_dispatcher_main(run_loop_sender, state, sender_resolver);
        });

        let sender = sender.wait()?;
        self.inner = Some(InnerIPCHandle { ipc_thread, sender });

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        *self.state.lock().unwrap() == IPCDispatcherState::Running
    }

    pub fn send_message(&self, message: ClientMessage) {
        if let Some(inner) = self.running_guard() {
            if let Err(err) = inner.sender.send(InternalMessage::SendIPC(message)) {
                log::warn!("Failed to send message to IPC dispatcher: {}", err);
            }
        }
    }

    fn running_guard(&self) -> Option<&InnerIPCHandle> {
        if self.is_running() {
            self.inner.as_ref()
        } else {
            None
        }
    }
}

struct InnerIPCHandle {
    ipc_thread: JoinHandle<()>,
    sender: mio_misc::channel::Sender<InternalMessage>,
}

enum InternalMessage {
    SendIPC(ClientMessage),
    Shutdown,
}
