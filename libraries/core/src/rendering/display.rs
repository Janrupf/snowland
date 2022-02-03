use std::fmt::Formatter;

use skia_safe::{Point, Rect};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Display {
    name: String,
    id: String,
    primary: bool,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Display {
    pub fn uninitialized() -> Self {
        Self {
            name: "Virtual uninitialized Display".into(),
            id: "UNINITIALIZED".into(),
            primary: false,
            x: -1,
            y: -1,
            width: -1,
            height: -1,
        }
    }

    pub fn new(
        name: String,
        id: String,
        primary: bool,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Self {
        Self {
            name,
            id,
            primary,
            x,
            y,
            width,
            height,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn primary(&self) -> bool {
        self.primary
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

    #[must_use = "remap_coordinates consumes and creates a new display instance without side effects"]
    pub fn remap_coordinates(self, new_x: i32, new_y: i32) -> Self {
        Self {
            x: new_x,
            y: new_y,
            ..self
        }
    }

    pub fn into_ipc(self) -> snowland_ipc::protocol::Display {
        snowland_ipc::protocol::Display {
            name: self.name,
            id: self.id,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            primary: self.primary,
        }
    }

    pub fn from_ipc(display: snowland_ipc::protocol::Display) -> Self {
        Self {
            name: display.name,
            id: display.id,
            x: display.x,
            y: display.y,
            width: display.width,
            height: display.height,
            primary: display.primary,
        }
    }
}

impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}x{}", self.name, self.width, self.height)
    }
}

impl From<Display> for snowland_ipc::protocol::Display {
    fn from(display: Display) -> Self {
        display.into_ipc()
    }
}

impl From<snowland_ipc::protocol::Display> for Display {
    fn from(display: snowland_ipc::protocol::Display) -> Self {
        Self::from_ipc(display)
    }
}
