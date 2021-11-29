use skia_safe::Canvas;

pub use xmas_countdown::*;

pub mod module;
mod xmas_countdown;

pub trait SnowlandScene {
    fn update(&mut self, canvas: &mut Canvas, width: u64, height: u64, delta: f32);
}

/// Generic description of the current scene.
#[derive(Debug)]
pub struct SceneData<'a> {
    canvas: &'a mut Canvas,
    width: u64,
    height: u64,
}

impl<'a> SceneData<'a> {
    pub fn new(canvas: &'a mut Canvas, width: u64, height: u64) -> Self {
        Self {
            canvas,
            width,
            height,
        }
    }

    /// The width of the scene in canvas units.
    fn width(&self) -> u64 {
        self.width
    }

    /// The height of the scene in canvas units.
    fn height(&self) -> u64 {
        self.height
    }

    /// The canvas the scene renders to.
    fn canvas(&mut self) -> &mut Canvas {
        self.canvas
    }
}
