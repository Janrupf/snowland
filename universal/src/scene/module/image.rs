use imgui::{InputText, TreeNodeFlags, Ui};
use native_dialog::FileDialog;
use serde::{Deserialize, Serialize};
use skia_safe::Image;

use crate::scene::module::part::{ModulePosition, PaintSetting};
use crate::scene::module::{Module, ModuleConfig, ModuleRenderer};
use crate::scene::SceneData;
use crate::ui::context::Context;
use crate::util::OwnedCodec;

pub(super) struct ImageModule;

impl Module for ImageModule {
    type Config = ImageModuleConfig;
    type Renderer = ImageModuleRenderer;

    fn create_renderer() -> Self::Renderer {
        ImageModuleRenderer::new()
    }

    fn name() -> String {
        "Image".into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageModuleConfig {
    position: ModulePosition,
    path: String,
    paint_enabled: bool,
    paint: PaintSetting,
}

impl Default for ImageModuleConfig {
    fn default() -> Self {
        Self {
            position: Default::default(),
            path: "".into(),
            paint_enabled: false,
            paint: PaintSetting::default(),
        }
    }
}

impl ModuleConfig for ImageModuleConfig {
    fn represent(&mut self, ui: &Ui, ctx: &Context<'_>) {
        if ui.collapsing_header("Position", TreeNodeFlags::FRAMED) {
            self.position.represent(ui, ctx);
        }

        if ui.collapsing_header("Module", TreeNodeFlags::FRAMED) {
            if let Some(_tok) = ui.begin_table("Module Options", 2) {
                ui.table_next_row();
                ui.table_next_column();

                InputText::new(ui, "Path", &mut self.path).build();
                ui.table_next_column();
                if ui.small_button("...") {
                    match FileDialog::new().show_open_single_file() {
                        Ok(None) => {}
                        Ok(Some(p)) => self.path = p.to_string_lossy().into(),
                        Err(err) => {
                            log::error!("Failed to show a file dialog: {}", err)
                        }
                    };
                }
            }
        }

        if ui.collapsing_header("Paint", TreeNodeFlags::FRAMED) {
            ui.checkbox("Enable paint override", &mut self.paint_enabled);

            if self.paint_enabled {
                self.paint.represent(ui, ctx);
            }
        }
    }
}

pub struct ImageModuleRenderer {
    current_path: String,
    current_image: Option<Image>,
}

impl ImageModuleRenderer {
    pub fn new() -> Self {
        Self {
            current_path: "".into(),
            current_image: None,
        }
    }
}

impl ModuleRenderer for ImageModuleRenderer {
    type Config = ImageModuleConfig;

    fn render<'a>(&mut self, config: &Self::Config, data: &mut SceneData<'a>) {
        if self.current_path != config.path {
            self.current_image = None;
            self.current_path = config.path.clone();

            if self.current_path.is_empty() {
                return;
            }

            let mut codec = match std::fs::read(&self.current_path).map(OwnedCodec::new) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    log::error!(
                        "Failed to decode image from \"{}\" as no decoder is available",
                        self.current_path
                    );
                    return;
                }
                Err(err) => {
                    log::error!(
                        "Failed to read image from \"{}\": {}",
                        self.current_path,
                        err
                    );
                    return;
                }
            };

            let image = match codec.get_image(None, None) {
                Ok(image) => image,
                Err(err) => {
                    log::error!(
                        "Failed to decode image from \"{}\": {:?}",
                        self.current_path,
                        err
                    );
                    return;
                }
            };

            self.current_image = Some(image);
        }

        if let Some(image) = &self.current_image {
            let pos = config.position.compute_position(
                data.width(),
                data.height(),
                image.width(),
                image.height(),
            );

            let canvas = data.canvas();
            let paint = config.paint_enabled.then(|| config.paint.get_paint());
            canvas.draw_image(image, pos, paint);
        }
    }
}
