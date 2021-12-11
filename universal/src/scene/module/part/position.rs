use imgui::{Drag, Ui};

use crate::scene::module::ModuleConfig;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HorizontalPositionAnchor {
    Left,
    Middle,
    Right,
}

impl HorizontalPositionAnchor {
    const VALUES: [Self; 3] = [Self::Left, Self::Middle, Self::Right];

    pub fn compute(&self, available: i32, value: i32) -> i32 {
        match self {
            Self::Left => 0,
            Self::Middle => (available / 2) - (value / 2),
            Self::Right => available - value,
        }
    }
}

impl Default for HorizontalPositionAnchor {
    fn default() -> Self {
        Self::Middle
    }
}

impl ModuleConfig for HorizontalPositionAnchor {
    fn represent(&mut self, ui: &Ui) {
        let mut current = Self::VALUES.iter().position(|v| v == self).unwrap();

        ui.combo("Horizontal", &mut current, &Self::VALUES, |v| {
            match v {
                Self::Left => "Left",
                Self::Middle => "Middle",
                Self::Right => "Right",
            }
            .into()
        });

        *self = Self::VALUES[current].clone();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerticalPositionAnchor {
    Top,
    Middle,
    Bottom,
}

impl VerticalPositionAnchor {
    const VALUES: [Self; 3] = [Self::Top, Self::Middle, Self::Bottom];

    pub fn compute(&self, available: i32, value: i32) -> i32 {
        match self {
            Self::Top => 0,
            Self::Middle => (available / 2) - (value / 2),
            Self::Bottom => available - value,
        }
    }

    pub fn compute_baselined(&self, available: i32, value: i32) -> i32 {
        match self {
            Self::Top => value,
            Self::Middle => (available / 2) + (value / 2),
            Self::Bottom => available,
        }
    }
}

impl Default for VerticalPositionAnchor {
    fn default() -> Self {
        Self::Middle
    }
}

impl ModuleConfig for VerticalPositionAnchor {
    fn represent(&mut self, ui: &Ui) {
        let mut current = Self::VALUES.iter().position(|v| v == self).unwrap();

        ui.combo("Vertical", &mut current, &Self::VALUES, |v| {
            match v {
                Self::Top => "Top",
                Self::Middle => "Middle",
                Self::Bottom => "Bottom",
            }
            .into()
        });

        *self = Self::VALUES[current].clone();
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ModulePosition {
    horizontal: HorizontalPositionAnchor,
    vertical: VerticalPositionAnchor,
    x_offset: i32,
    y_offset: i32,
}

impl ModulePosition {
    pub fn compute_position(
        &self,
        available_width: i32,
        available_height: i32,
        width: i32,
        height: i32,
    ) -> (i32, i32) {
        (
            self.horizontal.compute(available_width, width) + self.x_offset,
            self.vertical.compute(available_height, height) + self.y_offset,
        )
    }

    pub fn compute_position_baselined(
        &self,
        available_width: i32,
        available_height: i32,
        width: i32,
        height: i32,
    ) -> (i32, i32) {
        (
            self.horizontal.compute(available_width, width) + self.x_offset,
            self.vertical.compute_baselined(available_height, height) + self.y_offset,
        )
    }
}

impl ModuleConfig for ModulePosition {
    fn represent(&mut self, ui: &Ui) {
        if let Some(_tab) = ui.begin_table("Position", 2) {
            ui.table_next_row();
            ui.table_next_column();

            self.horizontal.represent(ui);
            ui.table_next_column();
            self.vertical.represent(ui);

            ui.table_next_row();
            ui.table_next_column();

            Drag::new("X Offset").build(ui, &mut self.x_offset);
            ui.table_next_column();
            Drag::new("Y Offset").build(ui, &mut self.y_offset);
        }
    }
}