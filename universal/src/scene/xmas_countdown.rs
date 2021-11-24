use rand::rngs::ThreadRng;
use rand::Rng;
use skia_safe::{Canvas, Color4f, Paint};

use crate::scene::SnowlandScene;

pub struct XMasCountdown {
    flakes: Vec<Snowflake>,
    random: ThreadRng,
    target_flake_count: u32,
}

impl XMasCountdown {
    pub fn new() -> Self {
        let random = rand::thread_rng();

        Self {
            flakes: Vec::new(),
            random,
            target_flake_count: 400,
        }
    }
}

impl SnowlandScene for XMasCountdown {
    fn update(&mut self, canvas: &mut Canvas, width: u64, height: u64, delta: f32) {
        log::debug!("delta = {}", delta);

        canvas.clear(Color4f::new(0.102, 0.102, 0.102, 1.0));

        if (self.target_flake_count as usize) != self.flakes.len() {
            self.flakes
                .resize_with(self.target_flake_count as usize, || {
                    Snowflake::new_random(width, height, &mut self.random)
                });
        }

        for flake in self.flakes.iter_mut() {
            flake.tick(canvas, delta, width, height, &mut self.random);
        }
    }
}

struct Snowflake {
    x: f32,
    y: f32,
    time_alive: f32,

    tumbling_multiplier: f32,
    time_to_live: f32,
    falling_speed: f32,
}

impl Snowflake {
    pub fn new_random(max_x: u64, max_y: u64, random: &mut ThreadRng) -> Self {
        let x = random.gen_range(0..max_x) as f32;
        let y = random.gen_range(0..max_y) as f32;

        let tumbling_multiplier = random.gen_range(0.0..=1.0);
        let time_to_live = random.gen_range(2000.0..=4000.0);
        let falling_speed = random.gen_range(1.0..=3.0);

        Self {
            x,
            y,
            time_alive: 0.0,
            tumbling_multiplier,
            time_to_live,
            falling_speed,
        }
    }

    pub fn tick(
        &mut self,
        canvas: &mut Canvas,
        delta: f32,
        x_limit: u64,
        y_limit: u64,
        random: &mut ThreadRng,
    ) {
        let mut paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, self.calculate_opacity()), None);
        paint.set_anti_alias(true);

        let tumble =
            ((self.time_alive / 1000.0).sin() - 0.5) * self.tumbling_multiplier * (delta / 20.0);
        let fall = self.falling_speed * (delta / 20.0);

        self.x += tumble;
        self.y += fall;

        canvas.draw_circle((self.x, self.y), 2.5, &paint);

        self.time_alive += delta;

        if self.time_alive > self.time_to_live
            || self.x < -10.0
            || self.x > (x_limit + 10) as f32
            || self.y > (y_limit + 10) as f32
        {
            self.reset(x_limit, y_limit, random);
        }
    }

    fn calculate_opacity(&self) -> f32 {
        f32::max(
            0.0,
            f32::min(
                f32::min(self.time_alive, self.time_to_live - self.time_alive),
                2000.0,
            ),
        ) / 2000.0
    }

    fn reset(&mut self, max_x: u64, max_y: u64, random: &mut ThreadRng) {
        *self = Self::new_random(max_x, max_y, random);
    }
}
