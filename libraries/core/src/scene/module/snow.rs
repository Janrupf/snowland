use rand::rngs::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use skia_safe::{Color4f, Paint};

use crate::scene::module::{Module, ModuleConfig, ModuleRenderer};
use crate::scene::SceneData;

pub(super) struct SnowModule;

impl Module for SnowModule {
    type Config = SnowModuleConfig;
    type Renderer = SnowModuleRenderer;

    fn create_renderer() -> Self::Renderer {
        SnowModuleRenderer::new()
    }

    fn name() -> String {
        "Snow".into()
    }
}

/// 400 flakes look good on 1920 * 1080.
const DEFAULT_PIXEL_FLAKE_RATIO: i32 = (1920 * 1080) / 400;
const DEFAULT_FADE_TIME: f32 = 2000.0;

const DEFAULT_TIME_TO_LIVE_MIN: f32 = 2000.0;
const DEFAULT_TIME_TO_LIVE_MAX: f32 = 4000.0;

const DEFAULT_TUMBLING_MIN: f32 = 0.0;
const DEFAULT_TUMBLING_MAX: f32 = 1.0;

const DEFAULT_FALLING_SPEED_MIN: f32 = 1.0;
const DEFAULT_FALLING_SPEED_MAX: f32 = 3.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnowModuleConfig {
    pixel_flake_ratio: i32,
    fade_time: f32,
    time_to_live_min: f32,
    time_to_live_max: f32,
    tumbling_min: f32,
    tumbling_max: f32,
    falling_speed_min: f32,
    falling_speed_max: f32,
}

impl Default for SnowModuleConfig {
    fn default() -> Self {
        Self {
            pixel_flake_ratio: DEFAULT_PIXEL_FLAKE_RATIO,
            fade_time: DEFAULT_FADE_TIME,
            time_to_live_min: DEFAULT_TIME_TO_LIVE_MIN,
            time_to_live_max: DEFAULT_TIME_TO_LIVE_MAX,
            tumbling_min: DEFAULT_TUMBLING_MIN,
            tumbling_max: DEFAULT_TUMBLING_MAX,
            falling_speed_min: DEFAULT_FALLING_SPEED_MIN,
            falling_speed_max: DEFAULT_FALLING_SPEED_MAX,
        }
    }
}

impl ModuleConfig for SnowModuleConfig {}

pub struct SnowModuleRenderer {
    flakes: Vec<Snowflake>,
}

impl SnowModuleRenderer {
    pub fn new() -> SnowModuleRenderer {
        Self { flakes: Vec::new() }
    }
}

impl ModuleRenderer for SnowModuleRenderer {
    type Config = SnowModuleConfig;

    fn render<'a>(&mut self, config: &Self::Config, data: &mut SceneData<'a>) {
        let target_flake_count = (data.width() * data.height()) / config.pixel_flake_ratio;

        let mut rng = ThreadRng::default();

        if (target_flake_count as usize) != self.flakes.len() {
            self.flakes.resize_with(target_flake_count as _, || {
                Snowflake::new_random(data, config, &mut rng)
            });
        }

        for flake in self.flakes.iter_mut() {
            flake.tick(data, config, &mut rng);
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
    pub fn new_random<'a>(
        data: &SceneData<'a>,
        config: &SnowModuleConfig,
        random: &mut ThreadRng,
    ) -> Self {
        let x = random.gen_range(0..data.width()) as f32;
        let y = random.gen_range(0..data.height()) as f32;

        let tumbling_multiplier = random.gen_range(config.tumbling_min..=config.tumbling_max);
        let time_to_live = random.gen_range(config.time_to_live_min..=config.time_to_live_max);
        let falling_speed = random.gen_range(config.falling_speed_min..=config.falling_speed_max);

        Self {
            x,
            y,
            time_alive: 0.0,
            tumbling_multiplier,
            time_to_live,
            falling_speed,
        }
    }

    pub fn tick<'a>(
        &mut self,
        data: &mut SceneData<'a>,
        config: &SnowModuleConfig,
        random: &mut ThreadRng,
    ) {
        let mut paint = Paint::new(
            Color4f::new(1.0, 1.0, 1.0, self.calculate_opacity(config)),
            None,
        );
        paint.set_anti_alias(true);

        let delta = data.delta().as_millis() as f32;

        let tumble = (self.time_alive / 1000.0).sin() * self.tumbling_multiplier * (delta / 20.0);
        let fall = self.falling_speed * (delta / 20.0);

        self.x += tumble;
        self.y += fall;

        data.canvas().draw_circle((self.x, self.y), 2.5, &paint);

        self.time_alive += delta;

        if self.time_alive > self.time_to_live
            || self.x < -10.0
            || self.x > (data.width() + 10) as f32
            || self.y > (data.height() + 10) as f32
        {
            self.reset(data, config, random);
        }
    }

    fn calculate_opacity(&self, config: &SnowModuleConfig) -> f32 {
        f32::max(
            0.0,
            f32::min(
                f32::min(self.time_alive, self.time_to_live - self.time_alive),
                config.fade_time,
            ),
        ) / config.fade_time
    }

    fn reset<'a>(
        &mut self,
        data: &SceneData<'a>,
        config: &SnowModuleConfig,
        random: &mut ThreadRng,
    ) {
        *self = Self::new_random(data, config, random);
    }
}
