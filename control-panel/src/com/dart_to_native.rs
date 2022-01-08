use crate::com::CommunicationError;
use crate::mcr;
use nativeshell::codec::Value;
use nativeshell::shell::{ContextRef, EngineHandle, MethodChannel};
use nativeshell::Context;
use std::time::Duration;

pub struct DartToNativeChannel;

impl DartToNativeChannel {
    pub fn register(context: &ContextRef) -> MethodChannel {
        let instance = DartToNativeChannel {};

        MethodChannel::new(context.weak(), "snowland_dart_to_native", instance)
    }
}

#[mcr::method_channel_call_handler]
impl DartToNativeChannel {
    pub fn connect_to_ipc(#[engine] engine: EngineHandle) -> Result<(), CommunicationError> {
        log::debug!("Attempting to connect to IPC...");

        std::thread::sleep(Duration::from_secs(4));

        let context = Context::current().unwrap();

        let invoker = context
            .message_manager
            .borrow()
            .get_method_invoker(engine, "snowland_native_to_dart");

        invoker.call_method("set_connected", Value::Bool(true), |_| {})?;

        Ok(())
    }
}
