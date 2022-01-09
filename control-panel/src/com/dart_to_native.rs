use crate::ipc::{IPCDispatcherError, IPCHandle};
use crate::mcr;
use nativeshell::shell::{ContextRef, EngineHandle, MethodChannel};

pub struct DartToNativeChannel {
    ipc_handle: IPCHandle,
}

impl DartToNativeChannel {
    pub fn register(context: &ContextRef) -> MethodChannel {
        context.run_loop.borrow().new_sender();

        let ipc_handle = IPCHandle::new(context.run_loop.borrow().new_sender());

        let instance = DartToNativeChannel { ipc_handle };

        MethodChannel::new(context.weak(), "snowland_dart_to_native", instance)
    }
}

/// These log levels correspond to 2 things at the same time:
///
/// * the log level of the [`log`] crate - these are translated to [`log::Level`] instances
/// * the log level used by the flutter application - see [logger.dart](../../lib/logger.dart)
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
    pub fn connect_to_ipc(
        &mut self,
        #[engine] engine: EngineHandle,
    ) -> Result<(), IPCDispatcherError> {
        log::debug!("Attempting to connect to IPC...");

        self.ipc_handle.start_connecting(engine)?;

        Ok(())
    }

    /// This is used to dispatch log messages from dart to the Rust logger.
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
