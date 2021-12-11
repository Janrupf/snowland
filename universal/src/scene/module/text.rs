use imgui::{InputText, TreeNodeFlags};
use skia_safe::{Font, Point};

use crate::rendering::fonts;
use crate::scene::module::part::{ModulePosition, PaintSetting};
use crate::scene::module::{Module, ModuleConfig, ModuleRenderer};
use crate::scene::SceneData;

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

#[derive(Debug)]
pub struct TextModuleConfig {
    position: ModulePosition,
    value: String,
    paint: PaintSetting,
    font: Font,
}

impl Default for TextModuleConfig {
    fn default() -> Self {
        Self {
            position: Default::default(),
            value: String::from("Custom text"),
            paint: PaintSetting::default(),
            font: Font::from_typeface(
                fonts::load_embedded_font(fonts::Font::NotoSansMono),
                Some(32.0),
            ),
        }
    }
}

impl Clone for TextModuleConfig {
    fn clone(&self) -> Self {
        let position = self.position.clone();
        let value = self.value.clone();
        let paint = self.paint.clone();
        let font = Font::from_typeface_with_params(
            self.font.typeface_or_default(),
            self.font.size(),
            self.font.scale_x(),
            self.font.skew_x(),
        );

        Self {
            position,
            value,
            paint,
            font,
        }
    }
}

impl ModuleConfig for TextModuleConfig {
    fn represent(&mut self, ui: &imgui::Ui) {
        if ui.collapsing_header("Position", TreeNodeFlags::FRAMED) {
            self.position.represent(ui);
        }

        if ui.collapsing_header("Color", TreeNodeFlags::FRAMED) {
            self.paint.represent(ui);
        }

        if ui.collapsing_header("Module", TreeNodeFlags::FRAMED) {
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
            &config.font,
            config.paint.get_paint(),
        );
    }
}
