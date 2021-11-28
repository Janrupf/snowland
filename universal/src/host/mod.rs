use skia_safe::Surface;

/// Abstraction for the underlying platform host.
pub trait SnowlandHost: Sized + 'static {
    type Renderer: SnowlandRenderer;
    type RendererCreator: SnowlandRendererCreator<Self>;
    type Error: std::error::Error + Send;

    /// Creates a callback which can be invoked on another thread in order to create
    /// an async render loop.
    fn prepare_renderer(&mut self) -> Self::RendererCreator;
}

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

/// Helper type alias for the renderer error type of a specific host.
pub type RendererError<H> = <<H as SnowlandHost>::Renderer as SnowlandRenderer>::Error;

/// Helper type alias for a result of a type [`R`] and a host specific renderer error.
pub type RendererResult<R, H> = Result<R, RendererError<H>>;

/// Trait which describes how a renderer should by created.
pub trait SnowlandRendererCreator<H>: Send
where
    H: SnowlandHost,
{
    /// Creates the renderer.
    fn create(self) -> RendererResult<H::Renderer, H>;
}

/// The most simple case where the creator is simply a function pointer
impl<H> SnowlandRendererCreator<H> for fn() -> RendererResult<H::Renderer, H>
where
    H: SnowlandHost,
{
    fn create(self) -> RendererResult<H::Renderer, H> {
        self()
    }
}

/// Implementation of a renderer creator which doesn't require further information.
pub struct SimpleRendererCreator<H>
where
    H: SnowlandHost,
{
    creator: Box<dyn FnOnce() -> RendererResult<H::Renderer, H> + Send>,
}

impl<H> SimpleRendererCreator<H>
where
    H: SnowlandHost,
{
    /// Creates the creator using a specific factory function
    pub fn new<F>(creator: F) -> Self
    where
        F: FnOnce() -> RendererResult<H::Renderer, H> + Send + 'static,
    {
        Self {
            creator: Box::new(creator),
        }
    }
}

impl<H> SnowlandRendererCreator<H> for SimpleRendererCreator<H>
where
    H: SnowlandHost,
{
    fn create(self) -> RendererResult<H::Renderer, H> {
        (self.creator)()
    }
}
