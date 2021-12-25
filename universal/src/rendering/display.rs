use std::fmt::Formatter;

use skia_safe::{Point, Rect};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Display {
    name: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Display {
    pub fn new(name: String, x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            name,
            x,
            y,
            width,
            height,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn upper_left(&self) -> Point {
        Point::new(self.x as _, self.y as _)
    }

    pub fn rect(&self) -> Rect {
        Rect::new(
            self.x as _,
            self.y as _,
            (self.x + self.width) as _,
            (self.y + self.height) as _,
        )
    }
}

impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}x{}", self.name, self.width, self.height)
    }
}
