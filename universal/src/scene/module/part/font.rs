use imgui::Ui;
use serde::{Deserialize, Serialize};
use skia_safe::Font;

use crate::rendering::fonts;
use crate::scene::module::ModuleConfig;
use crate::ui::context::Context;

#[derive(Debug, Serialize, Deserialize)]
pub struct FontSetting {
    #[serde(skip, default = "make_default_font")]
    inner: Font,
}

impl Clone for FontSetting {
    fn clone(&self) -> Self {
        let font = Font::from_typeface_with_params(
            self.inner.typeface_or_default(),
            self.inner.size(),
            self.inner.scale_x(),
            self.inner.skew_x(),
        );

        Self { inner: font }
    }
}

fn make_default_font() -> Font {
    Font::from_typeface(
        fonts::load_embedded_font(fonts::Font::NotoSansMono),
        Some(32.0),
    )
}

impl FontSetting {
    pub fn new() -> Self {
        Self {
            inner: make_default_font(),
        }
    }

    pub fn get_font(&self) -> &Font {
        &self.inner
    }
}

impl AsRef<Font> for FontSetting {
    fn as_ref(&self) -> &Font {
        self.get_font()
    }
}

impl Default for FontSetting {
    fn default() -> Self {
        FontSetting::new()
    }
}

impl ModuleConfig for FontSetting {
    fn represent(&mut self, _ui: &Ui, _ctx: &Context<'_>) {}
}
