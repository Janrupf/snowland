use chrono::{DateTime, Local, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use skia_safe::Point;

use crate::scene::module::part::{FontSetting, ModulePosition, PaintSetting};
use crate::scene::module::{Module, ModuleConfig, ModuleRenderer};
use crate::scene::SceneData;

pub(super) struct CountdownModule;

impl Module for CountdownModule {
    type Config = CountdownModuleConfig;
    type Renderer = CountdownModuleRenderer;

    fn create_renderer() -> Self::Renderer {
        CountdownModuleRenderer
    }

    fn name() -> String {
        "Countdown".into()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CountdownModuleConfig {
    position: ModulePosition,
    target: i64,
    paint: PaintSetting,
    font: FontSetting,
}

impl ModuleConfig for CountdownModuleConfig {}

pub struct CountdownModuleRenderer;

impl CountdownModuleRenderer {
    fn pluralize<'a>(value: i64, one: &'a str, other: &'a str) -> &'a str {
        match value {
            1 => one,
            _ => other,
        }
    }

    fn make_countdown_string(target: DateTime<Local>) -> String {
        let now = Local::now();
        let diff = target - now;

        let days = diff.num_days();
        let hours = diff.num_hours() % 24;
        let minutes = diff.num_minutes() % 60;
        let seconds = diff.num_seconds() % 60;

        format!(
            "{:0>3} {} {:0>2} {} {:0>2} {} and {:0>2} {}",
            days,
            Self::pluralize(days, "day, ", "days,"),
            hours,
            Self::pluralize(hours, "hour, ", "hours,"),
            minutes,
            Self::pluralize(minutes, "minute ", "minutes"),
            seconds,
            Self::pluralize(seconds, "second ", "seconds")
        )
    }
}

impl ModuleRenderer for CountdownModuleRenderer {
    type Config = CountdownModuleConfig;

    fn render<'a>(&mut self, config: &Self::Config, data: &mut SceneData<'a>) {
        let utc = Utc.timestamp_millis(config.target);
        let local = DateTime::<Local>::from(utc);

        let value = Self::make_countdown_string(local);

        let (_, rect) = config
            .font
            .get_font()
            .measure_str(&value, Some(config.paint.get_paint()));

        if let Some((x, y)) = config.position.compute_position_baselined(
            data,
            rect.width() as i32,
            rect.height() as i32,
        ) {
            let canvas = data.canvas();

            canvas.draw_str(
                &value,
                Point::new(x as _, y as _),
                config.font.get_font(),
                config.paint.get_paint(),
            );
        }
    }
}
