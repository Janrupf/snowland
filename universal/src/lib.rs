use crate::rendering::SnowlandRenderer;
use skia_safe::{Color4f, Paint, Rect, Surface};
use thiserror::Error;

pub mod rendering;

/// The heart of Snowland, application manager and central controller.
pub struct Snowland<R>
where
    R: SnowlandRenderer,
{
    renderer: R,
    surface: Option<Surface>,
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

        let canvas = surface.canvas();
        canvas.draw_rect(
            Rect::new(100.0, 100.0, 200.0, 200.0),
            &Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None),
        );

        surface.flush_and_submit();
        self.renderer.present().map_err(Error::PresentFailed)?;

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
}
