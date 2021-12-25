use imgui::{InputText, TreeNodeFlags};
use serde::{Deserialize, Serialize};
use skia_safe::Point;

use crate::scene::module::part::{FontSetting, ModulePosition, PaintSetting};
use crate::scene::module::{Module, ModuleConfig, ModuleRenderer};
use crate::scene::SceneData;
use crate::ui::context::Context;

pub(super) struct TextModule;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextModuleConfig {
    position: ModulePosition,
    value: String,
    paint: PaintSetting,
    font: FontSetting,
}

impl Default for TextModuleConfig {
    fn default() -> Self {
        Self {
            position: Default::default(),
            value: String::from("Custom text"),
            paint: PaintSetting::default(),
            font: FontSetting::default(),
        }
    }
}

impl ModuleConfig for TextModuleConfig {
    fn represent(&mut self, ui: &imgui::Ui, ctx: &Context<'_>) {
        if ui.collapsing_header("Position", TreeNodeFlags::FRAMED) {
            self.position.represent(ui, ctx);
        }

        if ui.collapsing_header("Color", TreeNodeFlags::FRAMED) {
            self.paint.represent(ui, ctx);
        }

        if ui.collapsing_header("Text", TreeNodeFlags::FRAMED) {
            InputText::new(ui, "Value", &mut self.value).build();
        }
    }
}

pub struct TextModuleRenderer;

impl ModuleRenderer for TextModuleRenderer {
    type Config = TextModuleConfig;

    fn render<'a>(&mut self, config: &Self::Config, data: &mut SceneData<'a>) {
        let (_, rect) = config
            .font
            .get_font()
            .measure_str(&config.value, Some(config.paint.get_paint()));

        let (x, y) = config.position.compute_position_baselined(
            data.width(),
            data.height(),
            rect.width() as i32,
            rect.height() as i32,
        );

        let canvas = data.canvas();

        canvas.draw_str(
            &config.value,
            Point::new(x as _, y as _),
            config.font.get_font(),
            config.paint.get_paint(),
        );
    }
}
