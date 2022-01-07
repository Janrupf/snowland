#![feature(drain_filter, once_cell)]

use crate::host::SnowlandRenderer;
use crate::rendering::RendererContainer;
use std::any::Any;
use std::time::SystemTimeError;
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
}

impl<R> Snowland<R>
where
    R: SnowlandRenderer,
{
    pub fn create(renderer: R) -> Result<Self, Error<R::Error>> {
        let container = RendererContainer::new(renderer).map_err(Error::RendererError)?;
        Ok(Self { container })
    }

    pub fn draw_frame(&mut self) -> Result<(), Error<R::Error>> {
        self.container.draw_frame().map_err(Error::RendererError)
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

    #[error("generic error: {description} ({cause:?})")]
    Generic {
        description: String,
        cause: Box<dyn Any + Send>,
    },
}
