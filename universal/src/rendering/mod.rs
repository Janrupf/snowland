use std::sync::Arc;
use std::time::Instant;

use skia_safe::Surface;

use crate::rendering::state::SharedRendererState;
use crate::{
    RendererError, SnowlandHost, SnowlandRenderer, SnowlandRendererCreator, SnowlandScene,
    XMasCountdown,
};

pub mod fonts;
pub mod state;

/// Contains the renderer and control over it.
pub struct RendererContainer<H>
where
    H: SnowlandHost,
{
    surface: Surface,
    renderer: H::Renderer,
    width: u64,
    height: u64,
    state: Arc<SharedRendererState>,
    last_frame_time: Instant,
    scene: Box<dyn SnowlandScene>,
}

impl<H> RendererContainer<H>
where
    H: SnowlandHost,
{
    /// Creates the container using a renderer creator.
    pub fn create_with(
        state: Arc<SharedRendererState>,
        creator: H::RendererCreator,
    ) -> Result<Self, RendererError<H>> {
        let mut renderer = creator.create()?;
        let (width, height) = renderer.get_size()?;
        let surface = renderer.create_surface(width, height)?;

        Ok(Self {
            renderer,
            surface,
            width,
            height,
            state,
            last_frame_time: Instant::now(),
            scene: Box::new(XMasCountdown::new()),
        })
    }

    /// Starts the run loop and renders frames.
    pub fn run(mut self) -> Result<(), RendererError<H>> {
        while !self.state.should_shutdown() {
            let (width, height) = self.renderer.get_size()?;
            self.resize(width, height)?;

            self.render_frame()?;
        }

        Ok(())
    }

    /// Resizes the internal surface if required.
    fn resize(&mut self, width: u64, height: u64) -> Result<(), RendererError<H>> {
        let needs_surface_recreation = self.width != width || self.height != height;

        if needs_surface_recreation {
            self.surface = self.renderer.create_surface(width, height)?;
        }

        Ok(())
    }

    /// Renders a single frame and ticks the scene.
    fn render_frame(&mut self) -> Result<(), RendererError<H>> {
        let width = self.surface.width();
        let height = self.surface.height();

        let canvas = self.surface.canvas();

        let last_frame_time = std::mem::replace(&mut self.last_frame_time, Instant::now());

        self.scene.update(
            canvas,
            width as u64,
            height as u64,
            (last_frame_time.elapsed().as_nanos() as f32) / 1000000.0,
        );

        self.surface.flush_and_submit();
        self.renderer.present()?;

        Ok(())
    }
}
