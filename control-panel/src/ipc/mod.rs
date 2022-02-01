mod dispatcher;
pub mod launcher;

pub use crate::ipc::dispatcher::IPCDispatcherError;
use crate::ipc::dispatcher::{ipc_dispatcher_main, IPCDispatcherState};
use nativeshell::shell::{EngineHandle, RunLoopSender};
use snowland_ipc::protocol::ClientMessage;
use snowland_misc::delayed::Delayed;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;

#[derive(Clone)]
pub struct IPCHandle {
    sender: RunLoopSender,
    state: Arc<Mutex<IPCDispatcherState>>,
    inner: Arc<RwLock<Option<InnerIPCHandle>>>,
}

impl IPCHandle {
    pub fn new(sender: RunLoopSender) -> Self {
        let state = Arc::new(Mutex::new(IPCDispatcherState::NotRunning));
        let inner = Arc::new(RwLock::new(None));

        Self {
            sender,
            state,
            inner,
        }
    }

    /// Starts connecting the IPC if it is not connected already.
    pub fn start_connecting(&mut self, engine: EngineHandle) -> Result<(), IPCDispatcherError> {
        if self.is_running() {
            return Ok(());
        }

        // Clone data that needs to be passed to the new thread
        let run_loop_sender = self.sender.clone();
        let state = self.state.clone();

        let (sender, sender_resolver) = Delayed::new();

        let handle = std::thread::spawn(move || {
            ipc_dispatcher_main(run_loop_sender, state, sender_resolver, engine);
        });

        let sender = sender.wait()?;
        *self.inner.write().unwrap() = Some(InnerIPCHandle { sender, handle });

        Ok(())
    }

    pub fn shutdown(&mut self) {
        let ok = self.running_guard(|inner| {
            if let Err(err) = inner.sender.send(InternalMessage::Shutdown) {
                log::warn!("Failed to send shutdown message to IPC dispatcher: {}", err);
            }
        });

        if ok {
            let mut guard = self.inner.write().unwrap();
            if let Some(inner) = guard.take() {
                if let Err(err) = inner.handle.join() {
                    log::warn!("Failed to join IPC dispatcher thread!");
                } else {
                    log::debug!("IPC dispatcher thread joined!");
                }
            }
        }
    }

    pub fn is_running(&self) -> bool {
        *self.state.lock().unwrap() == IPCDispatcherState::Running
    }

    pub fn send_message(&self, message: ClientMessage) {
        log::trace!("Handling message dispatch request...");

        let ok = self.running_guard(|inner| {
            if let Err(err) = inner.sender.send(InternalMessage::SendIPC(message)) {
                log::warn!("Failed to send message to IPC dispatcher: {}", err);
            }
        });

        if !ok {
            log::warn!("Tried to send IPC message while IPC was not running!");
        }
    }

    fn running_guard<F>(&self, callback: F) -> bool
    where
        F: FnOnce(&InnerIPCHandle),
    {
        if self.is_running() {
            let guard = self.inner.read().unwrap();

            if let Some(ref inner) = *guard {
                callback(inner);

                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

struct InnerIPCHandle {
    sender: mio_misc::channel::Sender<InternalMessage>,
    handle: JoinHandle<()>,
}

enum InternalMessage {
    SendIPC(ClientMessage),
    Shutdown,
}
