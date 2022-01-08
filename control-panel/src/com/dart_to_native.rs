use crate::com::CommunicationError;
use crate::mcr;
use nativeshell::codec::Value;
use nativeshell::shell::{ContextRef, EngineHandle, MethodChannel};
use nativeshell::Context;

pub struct DartToNativeChannel;

impl DartToNativeChannel {
    pub fn register(context: &ContextRef) -> MethodChannel {
        let instance = DartToNativeChannel {};

        MethodChannel::new(context.weak(), "snowland_dart_to_native", instance)
    }
}

#[derive(Debug, serde::Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn to_log_rs_level(&self) -> log::Level {
        match self {
            LogLevel::Trace => log::Level::Trace,
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Info => log::Level::Info,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Error => log::Level::Error,
        }
    }
}

#[mcr::method_channel_call_handler]
impl DartToNativeChannel {
    pub fn connect_to_ipc(#[engine] engine: EngineHandle) -> Result<(), CommunicationError> {
        log::debug!("Attempting to connect to IPC...");

        let context = Context::current().unwrap();

        let invoker = context
            .message_manager
            .borrow()
            .get_method_invoker(engine, "snowland_native_to_dart");

        invoker.call_method("set_connected", Value::Bool(true), |_| {})?;

        Ok(())
    }

    pub fn log(
        component: String,
        level: LogLevel,
        message: String,
    ) -> Result<(), std::convert::Infallible> {
        let level = level.to_log_rs_level();
        let target = format!("{{{{dart}}}}::{}", component);

        log::log!(target: &target, level, "{}", message);

        Ok(())
    }
}
