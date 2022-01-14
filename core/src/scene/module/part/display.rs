use serde::{Deserialize, Serialize};

use crate::scene::module::ModuleConfig;
use crate::scene::SceneData;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DisplaySelection {
    None,
    Primary,
    Identified { id: String, name: String },
}

impl DisplaySelection {
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

impl ModuleConfig for DisplaySelection {}
