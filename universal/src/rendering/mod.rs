use std::collections::HashMap;
use std::time::Instant;

use skia_safe::Surface;

use crate::rendering::display::Display;
use crate::scene::module::BoundModuleRenderer;
use crate::scene::SceneData;
use crate::SnowlandRenderer;

pub mod display;
pub mod fonts;
pub mod state;

/// Contains the renderer and control over it.
pub struct RendererContainer<R>
where
    R: SnowlandRenderer,
{
    surface: Surface,
    renderer: R,
    width: u64,
    height: u64,
    last_frame_time: Instant,
    modules: Vec<Box<dyn BoundModuleRenderer>>,
    primary_display: Display,
    displays: HashMap<String, Display>,
}

impl<R> RendererContainer<R>
where
    R: SnowlandRenderer,
{
    /// Creates the container using a renderer creator.
    pub fn new(mut renderer: R) -> Result<Self, R::Error> {
        let (width, height) = renderer.get_size()?;
        let surface = renderer.create_surface(width, height)?;

        Ok(Self {
            renderer,
            surface,
            width,
            height,
            last_frame_time: Instant::now(),
            modules: Vec::new(),
            primary_display: Display::uninitialized(),
            displays: HashMap::new(),
        })
    }

    /// Starts the run loop and renders frames.
    pub fn draw_frame(&mut self) -> Result<(), R::Error> {
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

        Ok(())
    }

    /// Resizes the internal surface if required.
    fn resize(&mut self, width: u64, height: u64) -> Result<(), R::Error> {
        let needs_surface_recreation = self.width != width || self.height != height;

        if needs_surface_recreation {
            self.surface = self.renderer.create_surface(width, height)?;
        }

        Ok(())
    }

    /// Renders a single frame and ticks the scene.
    fn render_frame(&mut self) -> Result<(), R::Error> {
        let width = self.surface.width();
        let height = self.surface.height();

        let canvas = self.surface.canvas();

        let last_frame_time = std::mem::replace(&mut self.last_frame_time, Instant::now());

        for module in &mut self.modules {
            let mut data = SceneData::new(
                canvas,
                &self.primary_display,
                &self.displays,
                width,
                height,
                last_frame_time.elapsed(),
            );
            module.render(&mut data);
        }

        self.surface.flush_and_submit();
        self.renderer.present()?;

        Ok(())
    }
}
