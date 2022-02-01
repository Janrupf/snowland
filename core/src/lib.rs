#![feature(drain_filter, once_cell)]

use crate::host::SnowlandRenderer;
use crate::io::ConfigIO;
use crate::rendering::display::Display;
use crate::rendering::RendererContainer;
use crate::scene::module::ModuleConfigError;
use snowland_ipc::protocol::ChangeConfiguration;
use snowland_ipc::protocol::{ClientMessage, Configuration, InstalledModule, ServerMessage};
use snowland_ipc::{SnowlandIPC, SnowlandIPCError};
use std::any::Any;
use std::ops::Add;
use std::time::{Duration, Instant, SystemTimeError};
use thiserror::Error;

pub mod control;
pub mod host;
pub mod io;
pub mod rendering;
mod scene;
pub mod util;

/// The heart of Snowland, application manager and central controller.
pub struct Snowland<R>
where
    R: SnowlandRenderer,
{
    container: RendererContainer<R>,
    ipc: SnowlandIPC<ServerMessage, ClientMessage>,
    next_scheduled_save: Option<Instant>,
}

impl<R> Snowland<R>
where
    R: SnowlandRenderer,
{
    /// Creates the snowland instance using the given renderer backend.
    pub fn create(renderer: R) -> Result<Self, Error<R::Error>> {
        let container = RendererContainer::new(renderer).map_err(Error::RendererError)?;
        let ipc = SnowlandIPC::create_server()?;
        Ok(Self {
            container,
            ipc,
            next_scheduled_save: None,
        })
    }

    /// Draws a frame using the underlying renderer.
    pub fn draw_frame(&mut self) -> Result<(), Error<R::Error>> {
        self.container.draw_frame().map_err(Error::RendererError)
    }

    /// Performs IPC related tasks.
    pub fn tick_ipc(&mut self) -> Result<(), Error<R::Error>> {
        if !self.ipc.is_connected() && self.ipc.nonblocking_accept()? {
            log::info!("IPC client connected!");
        }

        if self.ipc.is_connected() {
            if let Err(err) = self.ipc.nonblocking_write(ServerMessage::Heartbeat) {
                log::warn!("Failed to write IPC heartbeat: {}", err);
            } else {
                self.process_messages();
            }
        }

        Ok(())
    }

    /// Processes incoming messages and sends replies if required.
    fn process_messages(&mut self) {
        let messages = match self.ipc.nonblocking_read() {
            Ok(v) => v,
            Err(err) => {
                log::error!("Failed to read IPC messages: {}", err);
                return;
            }
        };

        if !messages.is_empty() {
            log::trace!("Received IPC messages = {:?}", messages);
        }

        let mut schedule_save = false;

        for message in messages {
            match message {
                ClientMessage::QueryConfiguration => self.send_configuration_over_ipc(),
                ClientMessage::QueryDisplays => self.send_displays_over_ipc(),
                ClientMessage::ReorderModules(old_index, new_index) => {
                    self.container.reorder_modules(old_index, new_index);
                    schedule_save = true;
                }
                ClientMessage::ChangeConfiguration(ChangeConfiguration {
                    module,
                    new_configuration,
                }) => {
                    log::trace!(
                        "Received configuration change request: {:#?}",
                        new_configuration
                    );
                    let new_configuration = new_configuration.into_json();

                    self.container
                        .replace_module_configuration(module, new_configuration);
                    schedule_save = true;
                }
                ClientMessage::AddModule(ty) => {
                    self.container.add_module_by_type(ty);
                    self.send_configuration_over_ipc();
                    schedule_save = true;
                }
                ClientMessage::RemoveModule(module) => {
                    self.container.remove_module(module);
                    self.send_configuration_over_ipc();
                    schedule_save = true;
                }
            }
        }

        if schedule_save {
            self.schedule_save();
        } else if let Some(scheduled) = self.next_scheduled_save {
            if scheduled <= Instant::now() {
                // Save has been scheduled for earlier or now!
                self.save_config_now();
            }
        }
    }

    /// Collects details about the current snowland configuration and sends the details
    /// over IPC.
    fn send_configuration_over_ipc(&mut self) {
        let installed: Result<Vec<_>, ModuleConfigError> = self
            .container
            .get_modules()
            .iter()
            .map(|m| {
                Ok(InstalledModule {
                    ty: m.module_type(),
                    configuration: m.serialize_config()?.into(),
                })
            })
            .collect();

        match installed {
            Ok(v) => {
                let configuration = Configuration { modules: v };

                self.dispatch_message(ServerMessage::UpdateConfiguration(configuration));
            }
            Err(err) => {
                log::error!("Failed to serialize module configurations: {}", err);
            }
        }
    }

    /// Collects the available displays and sends the details over IPC.
    fn send_displays_over_ipc(&mut self) {
        let displays = self
            .container
            .get_displays()
            .cloned()
            .map(Into::into)
            .collect();

        self.dispatch_message(ServerMessage::UpdateDisplays(displays));
    }

    /// Dispatches an IPC message and logs in case of an error
    fn dispatch_message(&mut self, message: ServerMessage) {
        log::trace!("Dispatching IPC message {:#?}", message);
        if let Err(err) = self.ipc.nonblocking_write(message) {
            log::warn!("Failed to write IPC message: {}", err);
        }
    }

    /// Schedules a save in 500 milliseconds.
    fn schedule_save(&mut self) {
        let next_save = Instant::now().add(Duration::from_millis(500));
        self.next_scheduled_save = Some(next_save);
    }

    /// Saves the module configuration now, clearing the next scheduled save, if any.
    fn save_config_now(&mut self) {
        match ConfigIO::save(self.container.get_modules().iter()) {
            Ok(()) => {
                log::info!("Saved modules!");
            }
            Err(err) => {
                log::error!("Failed to save modules: {}", err);
            }
        }

        self.next_scheduled_save = None;
    }

    /// Updates the displays used by renderer.
    pub fn update_displays(&mut self, displays: Vec<Display>) {
        self.container.update_displays(displays);
    }

    /// Loads the module configuration from disk.
    pub fn load_configuration_from_disk(&mut self) -> Result<(), ModuleConfigError> {
        log::info!("Loading module configuration from disk...");
        let modules = ConfigIO::load()?;
        self.container.replace_modules(modules);

        Ok(())
    }
}

impl<R> Drop for Snowland<R>
where
    R: SnowlandRenderer,
{
    fn drop(&mut self) {
        if self.next_scheduled_save.is_some() {
            self.save_config_now();
        }
    }
}

#[derive(Debug, Error)]
pub enum Error<R>
where
    R: std::error::Error,
{
    #[error("an I/O error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    RendererError(R),

    #[error("failed to calculate frame time: {0}")]
    TimeError(#[from] SystemTimeError),

    #[error("an error occurred on the ipc: {0}")]
    Ipc(#[from] SnowlandIPCError),

    #[error("generic error: {description} ({cause:?})")]
    Generic {
        description: String,
        cause: Box<dyn Any + Send>,
    },
}
