use crate::mcr;
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
    pub fn connect_to_ipc(#[engine] engine: EngineHandle) -> Result<(), std::convert::Infallible> {
        log::debug!("Attempting to connect to IPC...");

        std::thread::sleep(Duration::from_secs(4));

        let context = Context::current().unwrap();

        Ok(())
    }
}
