use crate::RendererController;
use egui::{CentralPanel, CtxRef, SidePanel};

use crate::ui::module_list::ModuleList;

pub struct EguiPanel {
    modules: ModuleList,
}

impl EguiPanel {
    pub fn new() -> Self {
        Self {
            modules: ModuleList::new(),
        }
    }

    pub fn run(&mut self, ctx: &CtxRef, controller: &RendererController) {
        SidePanel::left("Module list")
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Modules");
                });

                ui.separator();

                self.modules.render(ui);
            });

        CentralPanel::default().show(ctx, |ui| {});
    }
}
