mod xmas_countdown;
pub use xmas_countdown::*;

use skia_safe::Canvas;

pub trait SnowlandScene {
    fn update(&mut self, canvas: &mut Canvas, width: u64, height: u64, delta: f32);
}