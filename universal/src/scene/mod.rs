use std::time::Duration;

use skia_safe::Canvas;

pub mod module;

pub trait SnowlandScene {
    fn update(&mut self, canvas: &mut Canvas, width: u64, height: u64, delta: f32);
}

/// Generic description of the current scene.
#[derive(Debug)]
pub struct SceneData<'a> {
    canvas: &'a mut Canvas,
    width: i32,
    height: i32,
    delta: Duration,
}

impl<'a> SceneData<'a> {
    pub fn new(canvas: &'a mut Canvas, width: i32, height: i32, delta: Duration) -> Self {
        Self {
            canvas,
            width,
            height,
            delta,
        }
    }

    /// The width of the scene in canvas units.
    fn width(&self) -> i32 {
        self.width
    }

    /// The height of the scene in canvas units.
    fn height(&self) -> i32 {
        self.height
    }

    /// The canvas the scene renders to.
    fn canvas(&mut self) -> &mut Canvas {
        self.canvas
    }

    /// The rendering delta.
    fn delta(&self) -> &Duration {
        &self.delta
    }
}
