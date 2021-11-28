#![feature(drain_filter)]

use std::any::Any;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::SystemTimeError;

use thiserror::Error;

use crate::control::ControlMessage;
use crate::host::{RendererError, SnowlandHost, SnowlandRenderer, SnowlandRendererCreator};
use crate::rendering::state::SharedRendererState;
use crate::rendering::RendererContainer;
use crate::scene::{SnowlandScene, XMasCountdown};
use crate::ui::SnowlandUI;
use crate::util::{Delayed, Notifier};

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
    notifier: Notifier<ControlMessage>,
    state: Arc<SharedRendererState>,
}

impl<H> Snowland<H>
where
    H: SnowlandHost,
{
    pub fn create_with<F>(creator: F) -> Result<Self, Error<H>>
    where
        F: FnOnce(Notifier<ControlMessage>) -> Result<(H, Notifier<ControlMessage>), H::Error>,
    {
        let (ui, notifier) = SnowlandUI::new()?;
        let (host, notifier) = creator(notifier).map_err(Error::HostError)?;

        Ok(Self {
            ui,
            host,
            notifier,
            state: SharedRendererState::new(),
        })
    }

    /// Starts the snowland run loop.
    pub fn run(mut self) -> Result<(), Error<H>> {
        let renderer_creator = self.host.prepare_renderer();
        let renderer_handle = self.create_renderer_thread(renderer_creator)?;

        let ui_result = self.ui.run_loop(&self.notifier);

        self.state.initiate_shutdown();
        renderer_handle.join().map_err(|err| Error::<H>::Generic {
            description: "failed to join renderer thread".into(),
            cause: err,
        })?;

        ui_result.map_err(Error::from)
    }

    fn create_renderer_thread(
        &mut self,
        creator: H::RendererCreator,
    ) -> Result<JoinHandle<()>, Error<H>> {
        let (delayed, resolver) = Delayed::new();

        let state = self.state.clone();

        let join_handle = std::thread::Builder::new()
            .name("Renderer".into())
            .spawn(move || {
                let container = match RendererContainer::<H>::create_with(state, creator) {
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
