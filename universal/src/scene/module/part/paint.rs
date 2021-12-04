use imgui::{ColorEdit, EditableColor, Slider, SliderFlags, Ui};
use skia_safe::{paint::Style, Color4f, Paint};

use crate::scene::module::ModuleConfig;

#[derive(Debug, Clone)]
pub struct ColorSetting(Color4f);

impl ColorSetting {
    pub fn get_color(&self) -> Color4f {
        self.0
    }
}

impl Default for ColorSetting {
    fn default() -> Self {
        let color = Color4f::new(1.0, 1.0, 1.0, 1.0);
        Self(color)
    }
}

impl ModuleConfig for ColorSetting {
    fn represent(&mut self, ui: &Ui) {
        let color_data = self.0.as_array_mut();
        ColorEdit::new("Color", EditableColor::Float4(color_data)).build(ui);
    }
}

impl From<ColorSetting> for Color4f {
    fn from(setting: ColorSetting) -> Self {
        setting.0
    }
}

#[derive(Debug, Clone)]
pub struct PaintSetting(Paint);

impl PaintSetting {
    pub fn get_paint(&self) -> &Paint {
        &self.0
    }
}

impl Default for PaintSetting {
    fn default() -> Self {
        let paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        Self(paint)
    }
}

impl ModuleConfig for PaintSetting {
    fn represent(&mut self, ui: &Ui) {
        let mut color = self.0.color4f();
        let color_data = color.as_array_mut();

        if ColorEdit::new("Color", EditableColor::Float4(color_data)).build(ui) {
            self.0.set_color4f(
                unsafe { std::mem::transmute_copy::<_, Color4f>(color_data) },
                None,
            );
        }

        let mut anti_alias = self.0.is_anti_alias();
        if ui.checkbox("Anti alias", &mut anti_alias) {
            self.0.set_anti_alias(anti_alias);
        }

        let mut dither = self.0.is_dither();
        if ui.checkbox("Dither", &mut dither) {
            self.0.set_dither(dither);
        }

        let style = self.0.style();
        let mut stroked = style == Style::Stroke;
        if ui.checkbox("Stroke", &mut stroked) {
            self.0.set_stroke(stroked);
        }

        if stroked {
            let mut stroke_width = self.0.stroke_width();
            if Slider::new("Stroke width", 0.0, 100.0)
                .display_format("%.0f")
                .build(ui, &mut stroke_width)
            {
                log::debug!("Setting width {}", stroke_width);
                self.0.set_stroke_width(stroke_width);
            }

            let mut stroke_miter = self.0.stroke_miter();
            if Slider::new("Stroke miter", 0.0, 100.0)
                .display_format("%.0f")
                .build(ui, &mut stroke_miter)
            {
                self.0.set_stroke_miter(stroke_miter);
            }
        }
    }
}

impl From<PaintSetting> for Paint {
    fn from(setting: PaintSetting) -> Self {
        setting.0
    }
}
