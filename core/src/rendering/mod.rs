use skia_safe::luma_color_filter::new;
use std::collections::HashMap;
use std::time::Instant;

use skia_safe::Surface;

use crate::rendering::display::Display;
use crate::scene::module::ModuleContainer;
use crate::scene::SceneData;
use crate::SnowlandRenderer;

pub mod display;
pub mod fonts;

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
    modules: Vec<Box<dyn ModuleContainer>>,
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

        Ok(())
    }

    /// Updates the available displays.
    pub fn update_displays(&mut self, displays: Vec<Display>) {
        self.primary_display = displays
            .iter()
            .find(|d| d.primary())
            .or_else(|| displays.first())
            .map(Clone::clone)
            .unwrap_or_else(Display::uninitialized);

        self.displays = displays.into_iter().map(|d| (d.id().clone(), d)).collect();
    }

    /// Replaces the currently active list of modules.
    pub fn replace_modules(&mut self, modules: Vec<Box<dyn ModuleContainer>>) {
        self.modules = modules;
    }

    /// Retrieves the currently installed list of modules.
    pub fn get_modules(&self) -> &Vec<Box<dyn ModuleContainer>> {
        &self.modules
    }

    /// Changes the order of modules.
    pub fn reorder_modules(&mut self, old_index: usize, new_index: usize) {
        let m = self.modules.remove(old_index);
        self.modules.insert(new_index, m);
    }

    /// Updates the configuration of a module.
    pub fn replace_module_configuration(
        &mut self,
        index: usize,
        new_configuration: serde_json::Value,
    ) {
        if index > self.modules.len() {
            log::error!("Tried to update configuration for out of bounds module index {}, only {} modules are installed", index, self.modules.len());
            return;
        }

        if let Err(err) = self.modules[index].update_config(new_configuration) {
            log::error!("Failed to update module configuration: {}", err);
        }
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
            module.run_frame(&mut data);
        }

        self.surface.flush_and_submit();
        self.renderer.present()?;

        Ok(())
    }
}
