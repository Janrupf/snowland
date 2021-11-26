use crate::SnowlandRenderer;

pub trait SnowlandHost: Sized {
    type Renderer: SnowlandRenderer;
    type Error: std::error::Error;

    /// Retrieves the renderer used by the host.
    fn renderer(&mut self) -> &mut Self::Renderer;

    /// Called by snowland in order to give the host a chance to process events.
    fn process_messages(&mut self) -> Result<bool, Self::Error>;
    
    /// Retrieves the size of the area to be rendered.
    fn get_size(&self) -> Result<(u64, u64), Self::Error>;

    /// Flushes the frame and blocks the thread until the frame has been vertically synced.
    fn flush_frame(&mut self) -> Result<(), Self::Error>;
}
