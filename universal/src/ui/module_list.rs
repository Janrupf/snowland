use std::fmt::{Display, Formatter};

use egui::{Align, ComboBox, Layout, ScrollArea, Ui};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
enum ModuleType {
    Background,
    Text,
    Snow,
}

impl Display for ModuleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Background => write!(f, "Background"),
            Self::Text => write!(f, "Text"),
            Self::Snow => write!(f, "Snow"),
        }
    }
}

pub struct ModuleList {
    entries: Vec<ModuleEntry>,
    selected_add_type: ModuleType,
}

impl ModuleList {
    pub fn new() -> Self {
        Self {
            entries: vec![ModuleEntry::new(), ModuleEntry::new()],
            selected_add_type: ModuleType::Background,
        }
    }

    pub fn render(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ComboBox::from_label("Type")
                .selected_text(self.selected_add_type.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.selected_add_type,
                        ModuleType::Background,
                        ModuleType::Background.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.selected_add_type,
                        ModuleType::Snow,
                        ModuleType::Snow.to_string(),
                    );
                    ui.selectable_value(
                        &mut self.selected_add_type,
                        ModuleType::Text,
                        ModuleType::Text.to_string(),
                    );
                });

            if ui.button("Add").clicked() {
                self.entries.push(ModuleEntry::new());
            }
        });

        ui.separator();

        ui.vertical(|ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.entries.drain_filter(|m| m.render(ui));
                });
        });
    }
}

impl Default for ModuleList {
    fn default() -> Self {
        Self::new()
    }
}

struct ModuleEntry;

impl ModuleEntry {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&mut self, ui: &mut Ui) -> bool {
        ui.horizontal(|ui| {
            if ui.button("-").clicked() {
                return true;
            }

            ui.button("*");

            ui.label("Module name");

            false
        })
        .inner
    }
}
