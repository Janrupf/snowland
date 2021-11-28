use thiserror::Error;
use windows::Win32::Graphics::Dwm::DwmFlush;

use snowland_universal::control::ControlMessage;
use snowland_universal::host::{RendererResult, SimpleRendererCreator, SnowlandHost};

use crate::shell::messenger::ReceiveResult;
use crate::{
    start_shell_integration, Graphics, HostMessenger, HostToIntegrationMessage, ProgMan,
    SkiaWGLSnowlandRender, WinApiError, Worker,
};

/// Win32 host implementation for snowland.
#[derive(Debug)]
pub struct WinHost {
    messenger: HostMessenger,
}

impl WinHost {
    /// Creates a new Win32 host implementation.
    pub fn new() -> Result<Self, Error> {
        let messenger = start_shell_integration();

        Ok(Self { messenger })
    }
}

impl SnowlandHost for WinHost {
    type Renderer = SkiaWGLSnowlandRender;
    type RendererCreator = fn() -> RendererResult<Self::Renderer, Self>;
    type Error = Error;

    fn prepare_renderer(&mut self) -> Self::RendererCreator {
        SkiaWGLSnowlandRender::init
    }

    fn process_messages(
        &mut self,
        control_messages: &[ControlMessage],
    ) -> Result<Vec<ControlMessage>, Self::Error> {
        for message in control_messages {
            self.messenger
                .send(HostToIntegrationMessage::Control(message.clone()));
        }

        let mut messages = Vec::new();

        loop {
            let received = self.messenger.receive();
            let received = match received {
                ReceiveResult::Message(msg) => msg,
                ReceiveResult::None => break,
                ReceiveResult::Shutdown => return Ok(vec![ControlMessage::Exit]),
            };

            messages.push(received)
        }

        Ok(messages)
    }
}

impl Drop for WinHost {
    fn drop(&mut self) {
        self.messenger.send(HostToIntegrationMessage::QuitLoop);
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("the renderer failed to perform an operation: {0}")]
    Renderer(#[from] crate::graphics::SkiaWGLError),
}
