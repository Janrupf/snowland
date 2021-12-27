mod atom;
mod display;
mod drawable;
mod gc;
mod pixmap;
mod screen;
mod visual;
mod window;

pub use atom::*;
pub use display::*;
pub use drawable::*;
pub use gc::*;
pub use pixmap::*;
pub use screen::*;
pub use visual::*;
pub use window::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum XLibError {
    #[error("failed to open display")]
    OpenDisplayFailed,
}
