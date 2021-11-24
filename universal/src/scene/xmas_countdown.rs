use std::time::SystemTime;

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rand::rngs::ThreadRng;
use rand::Rng;
use skia_safe::{scalar, Canvas, Color4f, Font, Paint, Typeface};

use crate::rendering::fonts;
use crate::scene::SnowlandScene;

pub struct XMasCountdown {
    flakes: Vec<Snowflake>,
    random: ThreadRng,
    target_flake_count: u32,
    noto_sans_mono: Font,
}

impl XMasCountdown {
    pub fn new() -> Self {
        let random = rand::thread_rng();

        Self {
            flakes: Vec::new(),
            random,
            target_flake_count: 400,
            noto_sans_mono: Font::from_typeface(
                fonts::load_embedded_font(fonts::Font::NotoSansMono),
                Some(32.0),
            ),
        }
    }

    fn write_text_centered(
        &self,
        canvas: &mut Canvas,
        width: u64,
        height: u64,
        x_offset: f32,
        y_offset: f32,
        text: impl AsRef<str>,
        paint: &Paint,
    ) -> (scalar, scalar) {
        let (text_width, rect) = self.noto_sans_mono.measure_str(&text, Some(paint));

        let x = ((width / 2) as f32 - (text_width / 2.0)) + x_offset;
        let y = ((height / 2) as f32 - (rect.height() / 2.0)) + y_offset;

        canvas.draw_str(text, (x, y), &self.noto_sans_mono, paint);

        (rect.width(), rect.height())
    }

    fn pluralize<'a>(value: i64, one: &'a str, other: &'a str) -> &'a str {
        match value {
            1 => one,
            _ => other,
        }
    }

    fn time_until_christmas_message() -> String {
        let now = Local::now();
        let christmas = Local::today()
            .with_day(25)
            .and_then(|d| d.with_month(12))
            .and_then(|d| {
                if now.date() > d {
                    d.with_year(now.year() + 1)
                } else {
                    Some(d)
                }
            })
            .unwrap();

        let time_until_christmas = christmas.and_time(NaiveTime::from_hms(0, 0, 0)).unwrap() - now;

        let days = time_until_christmas.num_days();
        let hours = time_until_christmas.num_hours() % 24;
        let minutes = time_until_christmas.num_minutes() % 60;
        let seconds = time_until_christmas.num_seconds() % 60;

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

impl SnowlandScene for XMasCountdown {
    fn update(&mut self, canvas: &mut Canvas, width: u64, height: u64, delta: f32) {
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

        let mut font_paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        font_paint.set_anti_alias(true);

        let (_, main_height) = self.write_text_centered(
            canvas,
            width,
            height,
            0.0,
            0.0,
            Self::time_until_christmas_message(),
            &font_paint,
        );

        self.write_text_centered(
            canvas,
            width,
            height,
            0.0,
            -(main_height * 1.3),
            "There are only",
            &font_paint,
        );

        self.write_text_centered(
            canvas,
            width,
            height,
            0.0,
            main_height * 1.3,
            "left until christmas!",
            &font_paint,
        );
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
