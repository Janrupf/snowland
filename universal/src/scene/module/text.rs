use skia_safe::{Color4f, Font, Paint, Point};

use crate::scene::module::{Module, ModuleConfig, ModuleRenderer};
use crate::scene::SceneData;

pub(super) struct TextModule;

#[derive(Debug, Clone)]
pub struct TextModuleConfig {
    x: u64,
    y: u64,
    value: String,
}

impl Default for TextModuleConfig {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            value: String::from("Custom text"),
        }
    }
}

impl ModuleConfig for TextModuleConfig {
    fn represent(&mut self, ui: &imgui::Ui) {}
}

pub struct TextModuleRenderer;

impl ModuleRenderer for TextModuleRenderer {
    type Config = TextModuleConfig;

    fn render<'a>(&mut self, config: &Self::Config, data: &mut SceneData<'a>) {
        let canvas = data.canvas();

        canvas.draw_str(
            &config.value,
            Point::new(config.x as _, config.y as _),
            &Font::default(),
            &Paint::new(Color4f::from(0xFFFFFF), None),
        );
    }
}

impl Module for TextModule {
    type Config = TextModuleConfig;
    type Renderer = TextModuleRenderer;

    fn create_renderer() -> Self::Renderer {
        TextModuleRenderer
    }

    fn name() -> String {
        "Text".into()
    }
}
