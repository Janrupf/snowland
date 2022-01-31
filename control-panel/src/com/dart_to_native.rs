use crate::ipc::{IPCDispatcherError, IPCHandle};
use crate::mcr;
use nativeshell::shell::{ContextRef, EngineHandle, MethodChannel};
use snowland_ipc::protocol::{ChangeConfiguration, ClientMessage};

pub struct DartToNativeChannel {
    ipc_handle: IPCHandle,
}

impl DartToNativeChannel {
    pub fn register(context: &ContextRef, ipc_handle: IPCHandle) -> MethodChannel {
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

    /// Asks the IPC dispatcher to send a [`ClientMessage::QueryConfiguration`] message over IPC
    /// to the daemon.
    pub fn query_configuration(&mut self) -> Result<(), std::convert::Infallible> {
        self.ipc_handle
            .send_message(ClientMessage::QueryConfiguration);

        Ok(())
    }

    /// Asks the IPC dispatcher to send a [`ClientMessage::ReorderModules`] message over IPC to the
    /// daemon.
    pub fn reorder_modules(
        &mut self,
        old_index: usize,
        new_index: usize,
    ) -> Result<(), std::convert::Infallible> {
        self.ipc_handle
            .send_message(ClientMessage::ReorderModules(old_index, new_index));

        Ok(())
    }

    /// Asks the IPC dispatcher to send a [`ClientMessage::ChangeConfiguration`] message over IPC to
    /// the daemon.
    pub fn change_configuration(
        &mut self,
        module: usize,
        new_configuration: nativeshell::codec::Value,
    ) -> Result<(), std::convert::Infallible> {
        self.ipc_handle
            .send_message(ClientMessage::ChangeConfiguration(ChangeConfiguration {
                module,
                new_configuration: crate::util::value_to_structure(new_configuration),
            }));

        Ok(())
    }

    /// Asks the IPC dispatcher to send a [`ClientMessage::AddModule`] message over IPC to the
    /// daemon.
    pub fn add_module(&mut self, ty: String) -> Result<(), std::convert::Infallible> {
        self.ipc_handle.send_message(ClientMessage::AddModule(ty));

        Ok(())
    }
}
