use egui::{CentralPanel, CtxRef};

pub struct EguiPanel {}

impl EguiPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self, ctx: &CtxRef) {
        CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label("Hello, World!");
            });
        });
    }
}
