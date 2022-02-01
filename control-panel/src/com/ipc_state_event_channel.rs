use crate::ipc::IPCDispatcherError;
use crate::{mcr, IPCHandle};
use nativeshell::shell::{ContextRef, EngineHandle, MethodChannel};

pub struct IpcStateEventChannel {
    ipc_handle: IPCHandle,
}

impl IpcStateEventChannel {
    pub fn register(context: &ContextRef, ipc_handle: IPCHandle) -> MethodChannel {
        context.run_loop.borrow().new_sender();

        let instance = IpcStateEventChannel { ipc_handle };

        MethodChannel::new(context.weak(), "ipc_state_event", instance)
    }
}

#[mcr::method_channel_call_handler]
impl IpcStateEventChannel {
    /// Starts listening to IPC state events.
    ///
    /// In reality this does not perform any listening, but instead attempts
    /// to connect to the IPC.
    pub fn listen(&mut self, #[engine] engine: EngineHandle) -> Result<(), IPCDispatcherError> {
        log::debug!("Attempting to connect to IPC...");

        self.ipc_handle.start_connecting(engine)
    }

    pub fn cancel() -> Result<(), std::convert::Infallible> {
        Ok(())
    }
}
