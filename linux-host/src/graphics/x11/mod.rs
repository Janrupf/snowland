mod display;
mod gc;
mod screen;
mod visual;
mod window;

pub use display::*;
pub use gc::*;
pub use screen::*;
pub use visual::*;
pub use window::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum XLibError {
    #[error("failed to open display")]
    OpenDisplayFailed,
}
