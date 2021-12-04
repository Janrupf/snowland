use imgui::Ui;

use crate::scene::module::part::ColorSetting;
use crate::scene::module::{Module, ModuleConfig, ModuleRenderer};
use crate::scene::SceneData;

pub(super) struct ClearModule;

impl Module for ClearModule {
    type Config = ClearModuleConfig;
    type Renderer = ClearModuleRenderer;

    fn create_renderer() -> Self::Renderer {
        ClearModuleRenderer
    }

    fn name() -> String {
        "Clear".into()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ClearModuleConfig {
    color: ColorSetting,
}

impl ModuleConfig for ClearModuleConfig {
    fn represent(&mut self, ui: &Ui) {
        self.color.represent(ui);
    }
}

pub struct ClearModuleRenderer;

impl ModuleRenderer for ClearModuleRenderer {
    type Config = ClearModuleConfig;

    fn render<'a>(&mut self, config: &Self::Config, data: &mut SceneData<'a>) {
        let color = config.color.get_color();
        data.canvas().clear(color);
    }
}
