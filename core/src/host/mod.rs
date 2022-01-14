use skia_safe::Surface;

/// Abstraction for Snowland renderer backends.
pub trait SnowlandRenderer {
    type Error: std::error::Error + Send;

    /// Creates a new surface for the given width and height.
    ///
    /// A surface can generally be re-used until the width and height of the rendering target
    /// changes.
    fn create_surface(&mut self, width: u64, height: u64) -> Result<Surface, Self::Error>;

    /// Presents the rendered content.
    fn present(&self) -> Result<(), Self::Error>;

    /// Retrieves the size of the area to be rendered.
    fn get_size(&self) -> Result<(u64, u64), Self::Error>;
}
