use crate::ipc::IPCHandle;
use crate::mcr;
use nativeshell::shell::{ContextRef, MethodChannel};
use snowland_ipc::protocol::ClientMessage;

pub struct IpcDisplayEventChannel {
    ipc_handle: IPCHandle,
}

impl IpcDisplayEventChannel {
    pub fn register(context: &ContextRef, ipc_handle: IPCHandle) -> MethodChannel {
        context.run_loop.borrow().new_sender();

        let instance = IpcDisplayEventChannel { ipc_handle };

        MethodChannel::new(context.weak(), "ipc_display_event", instance)
    }
}

#[mcr::method_channel_call_handler]
impl IpcDisplayEventChannel {
    /// Starts listening to display events.
    ///
    /// In reality this does not perform any listening, but instead queries
    /// the daemon for displays.
    pub fn listen(&mut self) -> Result<(), std::convert::Infallible> {
        self.ipc_handle.send_message(ClientMessage::QueryDisplays);

        Ok(())
    }

    pub fn cancel() -> Result<(), std::convert::Infallible> {
        Ok(())
    }
}
