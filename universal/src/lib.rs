#![feature(drain_filter)]

use std::any::Any;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTimeError};

use thiserror::Error;

use crate::control::ControlMessage;
use crate::host::{RendererError, SnowlandHost, SnowlandRenderer, SnowlandRendererCreator};
use crate::rendering::RendererContainer;
use crate::scene::{SnowlandScene, XMasCountdown};
use crate::ui::SnowlandUI;
use crate::util::Delayed;

pub mod control;
pub mod host;
pub mod rendering;
mod scene;
mod ui;
pub mod util;

/// The heart of Snowland, application manager and central controller.
pub struct Snowland<H>
where
    H: SnowlandHost,
{
    ui: SnowlandUI,
    host: H,
}

impl<H> Snowland<H>
where
    H: SnowlandHost,
{
    /// Creates a new snowland by using the given host.
    pub fn new(host: H) -> Result<Self, Error<H>> {
        Ok(Self {
            ui: SnowlandUI::new()?,
            host,
        })
    }

    /// Starts the snowland run loop.
    pub fn run(mut self) -> Result<(), Error<H>> {
        let renderer_creator = self.host.prepare_renderer();
        let renderer_handle = self.create_renderer_thread(renderer_creator)?;

        let mut ui_control_messages = Vec::new();

        loop {
            if !ui_control_messages.is_empty() {
                log::debug!("Control messages to host: {:?}", ui_control_messages);
            }

            let host_control_messages = self
                .host
                .process_messages(&ui_control_messages)
                .map_err(Error::HostError)?;

            ui_control_messages = self.ui.tick(&host_control_messages);

            if !host_control_messages.is_empty() {
                log::debug!("Control messages to UI: {:?}", host_control_messages);
            }

            if self.process_control_messages(&ui_control_messages)
                || self.process_control_messages(&host_control_messages)
            {
                break;
            }

            std::thread::sleep(Duration::from_secs(1));
        }

        renderer_handle.join().map_err(|err| Error::<H>::Generic {
            description: "failed to join renderer thread".into(),
            cause: err,
        })?;
        Ok(())
    }

    fn process_control_messages(&mut self, messages: &[ControlMessage]) -> bool {
        messages.contains(&ControlMessage::Exit)
    }

    fn create_renderer_thread(
        &mut self,
        creator: H::RendererCreator,
    ) -> Result<JoinHandle<()>, Error<H>> {
        let (delayed, resolver) = Delayed::new();

        let join_handle = std::thread::Builder::new()
            .name("Renderer".into())
            .spawn(move || {
                let container = match RendererContainer::<H>::create_with(creator) {
                    Ok(v) => v,
                    Err(err) => {
                        resolver.resolve(Err(Error::RendererError(err)));
                        return;
                    }
                };

                resolver.resolve(Ok(()));
                container.run().unwrap(); // TODO: Handle this properly
            })?;

        delayed.wait()?;
        Ok(join_handle)
    }
}

#[derive(Debug, Error)]
pub enum Error<H>
where
    H: SnowlandHost,
{
    #[error("an I/O error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    HostError(H::Error),

    #[error(transparent)]
    RendererError(RendererError<H>),

    #[error("failed to calculate frame time: {0}")]
    TimeError(#[from] SystemTimeError),

    #[error("failed to call into user interface: {0}")]
    Ui(#[from] ui::Error),

    #[error("generic error: {description} ({cause:?})")]
    Generic {
        description: String,
        cause: Box<dyn Any + Send>,
    },
}
