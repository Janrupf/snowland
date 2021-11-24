use skia_safe::Canvas;

pub use xmas_countdown::*;

mod xmas_countdown;

pub trait SnowlandScene {
    fn update(&mut self, canvas: &mut Canvas, width: u64, height: u64, delta: f32);
}
