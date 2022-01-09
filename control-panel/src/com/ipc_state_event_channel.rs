use crate::mcr;
use nativeshell::shell::{ContextRef, MethodChannel};

pub struct IpcStateEventChannel;

impl IpcStateEventChannel {
    pub fn register(context: &ContextRef) -> MethodChannel {
        context.run_loop.borrow().new_sender();

        let instance = IpcStateEventChannel {};

        MethodChannel::new(context.weak(), "ipc_state_event", instance)
    }
}

#[mcr::method_channel_call_handler]
impl IpcStateEventChannel {
    pub fn listen() -> Result<(), std::convert::Infallible> {
        Ok(())
    }

    pub fn cancel() -> Result<(), std::convert::Infallible> {
        Ok(())
    }
}
