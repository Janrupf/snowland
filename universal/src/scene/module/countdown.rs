use std::fmt::{Display, Formatter};

use chrono::{DateTime, Datelike, Local, NaiveTime};
use imgui::{TreeNodeFlags, Ui};
use serde::{Deserialize, Serialize};
use skia_safe::Point;

use crate::scene::module::part::{FontSetting, ModulePosition, PaintSetting};
use crate::scene::module::{Module, ModuleConfig, ModuleRenderer};
use crate::scene::SceneData;
use crate::ui::context::Context;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CountdownTarget {
    Christmas,
    NewYear,
}

impl CountdownTarget {
    pub const fn name(&self) -> &'static str {
        match self {
            CountdownTarget::Christmas => "Christmas",
            CountdownTarget::NewYear => "New Year",
        }
    }

    pub const fn ordinal(&self) -> usize {
        match self {
            CountdownTarget::Christmas => 0,
            CountdownTarget::NewYear => 1,
        }
    }

    pub const fn from_ordinal(ordinal: usize) -> Self {
        match ordinal {
            0 => CountdownTarget::Christmas,
            1 => CountdownTarget::NewYear,
            _ => panic!("Invalid countdown target ordinal"),
        }
    }

    pub const fn names() -> [&'static str; 2] {
        [Self::from_ordinal(0).name(), Self::from_ordinal(1).name()]
    }

    pub fn get_date_time(&self) -> DateTime<Local> {
        let now = Local::now();

        match self {
            CountdownTarget::Christmas => Local::today()
                .with_day(25)
                .and_then(|d| d.with_month(12))
                .and_then(|d| {
                    if now.date() > d {
                        d.with_year(now.year() + 1)
                    } else {
                        Some(d)
                    }
                })
                .and_then(|d| d.and_time(NaiveTime::from_hms(0, 0, 0)))
                .unwrap(),

            CountdownTarget::NewYear => Local::today()
                .with_day(1)
                .and_then(|d| d.with_month(1))
                .and_then(|d| d.with_year(now.year() + 1))
                .and_then(|d| d.and_time(NaiveTime::from_hms(0, 0, 0)))
                .unwrap(),
        }
    }
}

impl Display for CountdownTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Default for CountdownTarget {
    fn default() -> Self {
        Self::NewYear
    }
}

impl ModuleConfig for CountdownTarget {
    fn represent(&mut self, _ui: &Ui, _ctx: &Context<'_>) {}
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CountdownModuleConfig {
    position: ModulePosition,
    target: CountdownTarget,
    paint: PaintSetting,
    font: FontSetting,
}

impl ModuleConfig for CountdownModuleConfig {
    fn represent(&mut self, ui: &imgui::Ui, ctx: &Context<'_>) {
        if ui.collapsing_header("Position", TreeNodeFlags::FRAMED) {
            self.position.represent(ui, ctx);
        }

        if ui.collapsing_header("Color", TreeNodeFlags::FRAMED) {
            self.paint.represent(ui, ctx);
        }

        if ui.collapsing_header("Module", TreeNodeFlags::FRAMED) {
            let mut current_type_ordinal = self.target.ordinal();
            if ui.combo_simple_string(
                "Type",
                &mut current_type_ordinal,
                CountdownTarget::names().as_slice(),
            ) {
                self.target = CountdownTarget::from_ordinal(current_type_ordinal);
            }

            self.target.represent(ui, ctx);
        }
    }
}

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
        let value = Self::make_countdown_string(config.target.get_date_time());

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
