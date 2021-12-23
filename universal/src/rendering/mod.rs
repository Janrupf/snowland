use std::sync::mpsc::Receiver;
use std::time::Instant;

use skia_safe::Surface;

use crate::rendering::state::RendererStateMessage;
use crate::scene::module::BoundModuleRenderer;
use crate::scene::SceneData;
use crate::{
    RendererError, SnowlandHost, SnowlandRenderer, SnowlandRendererCreator, SnowlandScene,
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
    message_receiver: Receiver<RendererStateMessage>,
    last_frame_time: Instant,
    modules: Vec<Box<dyn BoundModuleRenderer>>,
}

impl<H> RendererContainer<H>
where
    H: SnowlandHost,
{
    /// Creates the container using a renderer creator.
    pub fn create_with(
        message_receiver: Receiver<RendererStateMessage>,
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
            message_receiver,
            last_frame_time: Instant::now(),
            modules: Vec::new(),
        })
    }

    /// Starts the run loop and renders frames.
    pub fn run(mut self) -> Result<(), RendererError<H>> {
        loop {
            let (width, height) = self.renderer.get_size()?;
            self.resize(width, height)?;

            self.render_frame()?;

            self.modules.retain(|m| {
                let remove = m.should_remove();

                if remove {
                    log::debug!("Removing module as it expired!")
                }

                !remove
            });

            while let Ok(message) = self.message_receiver.try_recv() {
                match message {
                    RendererStateMessage::Shutdown => return Ok(()),
                    RendererStateMessage::InsertModule { module } => {
                        log::debug!("Inserting new module!");
                        self.modules.push(module);
                    }
                    RendererStateMessage::Swap(a, b) => {
                        log::debug!("Swapping module {} with {}", a, b);
                        self.modules.swap(a, b);
                    }
                }
            }
        }
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

        for module in &mut self.modules {
            let mut data = SceneData::new(canvas, width, height, last_frame_time.elapsed());
            module.render(&mut data);
        }

        self.surface.flush_and_submit();
        self.renderer.present()?;

        Ok(())
    }
}
