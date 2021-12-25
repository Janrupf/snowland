use imgui::{Selectable, Ui};
use serde::{Deserialize, Serialize};

use crate::scene::module::ModuleConfig;
use crate::scene::SceneData;
use crate::ui::context::Context;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DisplaySelection {
    None,
    Primary,
    Identified { id: String, name: String },
}

impl DisplaySelection {
    const NONE_LABEL: &'static str = "None";
    const PRIMARY_LABEL: &'static str = "Primary";

    fn get_value_label(&self) -> &str {
        match self {
            DisplaySelection::None => Self::NONE_LABEL,
            DisplaySelection::Primary => Self::PRIMARY_LABEL,
            DisplaySelection::Identified { name, .. } => name,
        }
    }

    fn selection_box(&mut self, new_value: Self, ui: &Ui) {
        let label = new_value.get_value_label();
        let is_selected = *self == new_value;

        if Selectable::new(label).selected(is_selected).build(ui) {
            *self = new_value;
        }

        if is_selected {
            ui.set_item_default_focus();
        }
    }

    fn named_selection_box(&mut self, new_name: &str, new_id: &str, ui: &Ui) {
        let label = new_name;
        let is_selected = if let Self::Identified { id, name } = self {
            let selected = id == new_id;

            if selected && name != new_name {
                *name = new_name.to_string();
            }

            selected
        } else {
            false
        };

        if Selectable::new(label).selected(is_selected).build(ui) {
            *self = Self::Identified {
                name: new_name.to_string(),
                id: new_id.to_string(),
            };
        }

        if is_selected {
            ui.set_item_default_focus();
        }
    }

    pub fn perform_calculation_with<F>(&self, data: &SceneData, calc: F) -> Option<(i32, i32)>
    where
        F: FnOnce(i32, i32) -> (i32, i32),
    {
        let (width, height, start_x, start_y) = match self {
            DisplaySelection::None => (data.width(), data.height(), 0, 0),
            DisplaySelection::Primary => {
                let display = data.primary_display();

                (display.width(), display.height(), display.x(), display.y())
            }
            DisplaySelection::Identified { id, .. } => match data.lookup_display(id) {
                None => return None,
                Some(d) => (d.width(), d.height(), d.x(), d.y()),
            },
        };

        let (x, y) = calc(width, height);

        Some((start_x + x, start_y + y))
    }
}

impl Default for DisplaySelection {
    fn default() -> Self {
        DisplaySelection::Primary
    }
}

impl ModuleConfig for DisplaySelection {
    fn represent(&mut self, ui: &Ui, ctx: &Context<'_>) {
        if let Some(_tok) = ui.begin_combo("Display", self.get_value_label()) {
            self.selection_box(Self::None, ui);
            self.selection_box(Self::Primary, ui);

            for display in ctx.displays() {
                self.named_selection_box(display.name(), display.id(), ui);
            }
        }
    }
}
