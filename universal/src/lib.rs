#![feature(drain_filter)]

use crate::rendering::SnowlandRenderer;
use crate::scene::{SnowlandScene, XMasCountdown};
use skia_safe::{Color4f, Paint, Rect, Surface};
use std::time::{Instant, SystemTime, SystemTimeError, UNIX_EPOCH};
use thiserror::Error;

pub mod rendering;
mod scene;

/// The heart of Snowland, application manager and central controller.
pub struct Snowland<R>
where
    R: SnowlandRenderer,
{
    renderer: R,
    surface: Option<Surface>,
    last_frame_time: Option<f32>,
    scene: Box<dyn SnowlandScene>,
}

impl<R> Snowland<R>
where
    R: SnowlandRenderer,
{
    /// Creates a new snowland for the given renderer.
    pub fn new(renderer: R) -> Self {
        Self {
            renderer,
            surface: None,
            last_frame_time: None,
            scene: Box::new(XMasCountdown::new()),
        }
    }

    pub fn resize(&mut self, width: u64, height: u64) -> Result<(), Error> {
        let needs_surface_recreation = self
            .surface
            .as_ref()
            .map(|s| s.width() as u64 == width && s.height() as u64 == height)
            .unwrap_or(true);

        if needs_surface_recreation {
            let new_surface = self
                .renderer
                .create_surface(width, height)
                .map_err(Error::SurfaceCreationError)?;
            self.surface.replace(new_surface);
        }

        Ok(())
    }

    pub fn render_frame(&mut self) -> Result<(), Error> {
        let surface = self.surface.as_mut().ok_or(Error::NoSurface)?;

        let width = surface.width();
        let height = surface.height();

        let canvas = surface.canvas();

        let pre_frame_time = Instant::now();

        self.scene.update(
            canvas,
            width as u64,
            height as u64,
            self.last_frame_time.unwrap_or(0.0),
        );

        surface.flush_and_submit();
        self.renderer.present().map_err(Error::PresentFailed)?;

        self.last_frame_time
            .replace((pre_frame_time.elapsed().as_nanos() as f32) / 1000000.0);

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("no surface to render to")]
    NoSurface,

    #[error("failed to create surface: {0}")]
    SurfaceCreationError(Box<dyn std::error::Error>),

    #[error("failed to present surface: {0}")]
    PresentFailed(Box<dyn std::error::Error>),

    #[error("failed to calculate frame time: {0}")]
    TimeError(#[from] SystemTimeError),
}
