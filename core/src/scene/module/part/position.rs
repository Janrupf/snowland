use serde::{Deserialize, Serialize};

use crate::scene::module::part::DisplaySelection;
use crate::scene::module::ModuleConfig;
use crate::scene::SceneData;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HorizontalPositionAnchor {
    Left,
    Center,
    Right,
}

impl HorizontalPositionAnchor {
    const VALUES: [Self; 3] = [Self::Left, Self::Center, Self::Right];

    pub fn compute(&self, available: i32, value: i32) -> i32 {
        match self {
            Self::Left => 0,
            Self::Center => (available / 2) - (value / 2),
            Self::Right => available - value,
        }
    }
}

impl Default for HorizontalPositionAnchor {
    fn default() -> Self {
        Self::Center
    }
}

impl ModuleConfig for HorizontalPositionAnchor {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum VerticalPositionAnchor {
    Top,
    Center,
    Bottom,
}

impl VerticalPositionAnchor {
    const VALUES: [Self; 3] = [Self::Top, Self::Center, Self::Bottom];

    pub fn compute(&self, available: i32, value: i32) -> i32 {
        match self {
            Self::Top => 0,
            Self::Center => (available / 2) - (value / 2),
            Self::Bottom => available - value,
        }
    }

    pub fn compute_baselined(&self, available: i32, value: i32) -> i32 {
        match self {
            Self::Top => value,
            Self::Center => (available / 2) + (value / 2),
            Self::Bottom => available,
        }
    }
}

impl Default for VerticalPositionAnchor {
    fn default() -> Self {
        Self::Center
    }
}

impl ModuleConfig for VerticalPositionAnchor {}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct ModulePosition {
    horizontal: HorizontalPositionAnchor,
    vertical: VerticalPositionAnchor,

    #[serde(default)]
    display: DisplaySelection,
    x_offset: i32,
    y_offset: i32,
}

impl ModulePosition {
    pub fn compute_position(
        &self,
        data: &SceneData,
        width: i32,
        height: i32,
    ) -> Option<(i32, i32)> {
        self.display
            .perform_calculation_with(data, |available_width, available_height| {
                (
                    self.horizontal.compute(available_width, width) + self.x_offset,
                    self.vertical.compute(available_height, height) + self.y_offset,
                )
            })
    }

    pub fn compute_position_baselined(
        &self,
        data: &SceneData,
        width: i32,
        height: i32,
    ) -> Option<(i32, i32)> {
        self.display
            .perform_calculation_with(data, |available_width, available_height| {
                (
                    self.horizontal.compute(available_width, width) + self.x_offset,
                    self.vertical.compute_baselined(available_height, height) + self.y_offset,
                )
            })
    }
}

impl ModuleConfig for ModulePosition {}
