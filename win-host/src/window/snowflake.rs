use rand::Rng;
use skulpin::skia_safe::*;

pub struct Snowflake {
    time_to_live: u32,
    fall_speed: u32,
    tumbling_multiplier: u32,

    x: f32,
    y: f32,
    time_alive: u32,
}

impl Snowflake {
    pub fn new(
        time_to_live: u32,
        fall_speed: u32,
        tumbling_multiplier: u32,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            time_to_live,
            fall_speed,
            tumbling_multiplier,
            x,
            y,
            time_alive: 0,
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas, delta: u32) {
        canvas.self.y += self.fall_speed;
        self.x = rand::thread_rng().gen_range::<f32>(0..100)
    }
}
