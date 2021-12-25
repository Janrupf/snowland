use std::collections::HashMap;
use std::time::Duration;

use skia_safe::Canvas;

use crate::rendering::display::Display;

pub mod module;

/// Generic description of the current scene.
#[derive(Debug)]
pub struct SceneData<'a> {
    canvas: &'a mut Canvas,
    displays: &'a HashMap<String, Display>,
    width: i32,
    height: i32,
    delta: Duration,
}

impl<'a> SceneData<'a> {
    pub fn new(
        canvas: &'a mut Canvas,
        displays: &'a HashMap<String, Display>,
        width: i32,
        height: i32,
        delta: Duration,
    ) -> Self {
        Self {
            canvas,
            displays,
            width,
            height,
            delta,
        }
    }

    /// The width of the scene in canvas units.
    pub fn width(&self) -> i32 {
        self.width
    }

    /// The height of the scene in canvas units.
    pub fn height(&self) -> i32 {
        self.height
    }

    /// The canvas the scene renders to.
    pub fn canvas(&mut self) -> &mut Canvas {
        self.canvas
    }

    /// The rendering delta.
    pub fn delta(&self) -> &Duration {
        &self.delta
    }

    /// Attempts to find a display by name.
    pub fn lookup_display(&self, name: &str) -> Option<&'a Display> {
        self.displays.get(name)
    }
}
