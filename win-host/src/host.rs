use std::thread::JoinHandle;

use thiserror::Error;

use snowland_universal::control::ControlMessage;
use snowland_universal::host::{RendererResult, SnowlandHost};
use snowland_universal::util::Notifier;

use crate::{start_shell_integration, SkiaWGLSnowlandRender};

/// Win32 host implementation for snowland.
pub struct WinHost {
    notifier: Notifier<ControlMessage>,
    joiner: Option<JoinHandle<Result<(), crate::shell::Error>>>,
}

impl WinHost {
    /// Creates a new Win32 host implementation.
    pub fn new(
        notifier: Notifier<ControlMessage>,
    ) -> Result<(Self, Notifier<ControlMessage>), Error> {
        let (notifier, joiner) = start_shell_integration(notifier);

        Ok((
            Self {
                notifier: notifier.clone(),
                joiner: Some(joiner),
            },
            notifier,
        ))
    }
}

impl SnowlandHost for WinHost {
    type Renderer = SkiaWGLSnowlandRender;
    type RendererCreator = fn() -> RendererResult<Self::Renderer, Self>;
    type Error = Error;

    fn prepare_renderer(&mut self) -> Self::RendererCreator {
        SkiaWGLSnowlandRender::init
    }
}

impl Drop for WinHost {
    fn drop(&mut self) {
        self.notifier.notify(ControlMessage::Exit);
        match self.joiner.take().unwrap().join() {
            Ok(Ok(())) => {}
            Ok(Err(err)) => log::warn!("Integration finished with an error: {}", err),
            Err(err) => std::panic::resume_unwind(err),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("the renderer failed to perform an operation: {0}")]
    Renderer(#[from] crate::graphics::SkiaWGLError),
}
