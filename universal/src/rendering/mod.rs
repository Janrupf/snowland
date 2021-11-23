use skia_safe::Surface;
use std::error::Error;

/// Abstraction for Snowland renderer backends.
pub trait SnowlandRenderer {
    /// Creates a new surface for the given width and height.
    ///
    /// A surface can generally be re-used until the width and height of the rendering target
    /// changes.
    fn create_surface(&mut self, width: u64, height: u64) -> Result<Surface, Box<dyn Error>>;

    /// Presents the rendered content.
    fn present(&self) -> Result<(), Box<dyn Error>>;
}
