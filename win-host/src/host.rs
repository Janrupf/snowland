use std::sync::mpsc::{SendError, TryRecvError};

use thiserror::Error;
use windows::Win32::Graphics::Dwm::DwmFlush;

use snowland_universal::control::message_pipe::MessagePipeEnd;
use snowland_universal::control::ControlMessage;
use snowland_universal::host::SnowlandHost;

use crate::shell::messenger::{IntegrationToHostMessage, ReceiveResult};
use crate::{
    start_shell_integration, Graphics, HostMessenger, HostToIntegrationMessage, ProgMan,
    SkiaWGLSnowlandRender, WinApiError, Worker,
};

/// Win32 host implementation for snowland.
#[derive(Debug)]
pub struct WinHost {
    renderer: SkiaWGLSnowlandRender,
    graphics: Graphics,
    worker: Worker,
    prog_man: ProgMan,
    snowland_pipe: MessagePipeEnd<ControlMessage>,
    messenger: HostMessenger,
}

impl WinHost {
    /// Creates a new Win32 host implementation.
    pub fn new(snowland_pipe: MessagePipeEnd<ControlMessage>) -> Result<Self, Error> {
        let messenger = start_shell_integration();

        let prog_man = ProgMan::new()?;
        let worker = prog_man.get_or_create_worker()?;
        let graphics = Graphics::from_window(worker.get_handle())?;
        let wgl = graphics.create_wgl_context()?;
        let renderer = SkiaWGLSnowlandRender::from_context(wgl)?;

        Ok(Self {
            renderer,
            graphics,
            worker,
            prog_man,
            snowland_pipe,
            messenger,
        })
    }
}

impl SnowlandHost for WinHost {
    type Renderer = SkiaWGLSnowlandRender;
    type Error = Error;

    fn renderer(&mut self) -> &mut Self::Renderer {
        &mut self.renderer
    }

    fn process_messages(&mut self) -> Result<bool, Self::Error> {
        match self.messenger.receive() {
            ReceiveResult::Message(msg) => match msg {
                IntegrationToHostMessage::StopRendering => return Ok(false),
                IntegrationToHostMessage::Control(msg) => match self.snowland_pipe.send(msg) {
                    Ok(()) => {}
                    Err(err) => return Err(Error::PipeClosed(PipeClosedError::Send(err))),
                },
            },
            ReceiveResult::None => {}
            ReceiveResult::Shutdown => return Ok(false),
        }

        match self.snowland_pipe.try_recv() {
            Ok(msg) => self.messenger.send(HostToIntegrationMessage::Control(msg)),
            Err(TryRecvError::Empty) => {}
            Err(v) => return Err(Error::PipeClosed(PipeClosedError::Receive(v))),
        }

        Ok(true)
    }

    fn get_size(&self) -> Result<(u64, u64), Self::Error> {
        Ok(self.worker.get_size()?)
    }

    fn flush_frame(&mut self) -> Result<(), Self::Error> {
        unsafe { DwmFlush() }?;

        Ok(())
    }
}

impl Drop for WinHost {
    fn drop(&mut self) {
        self.messenger.send(HostToIntegrationMessage::QuitLoop);
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to create ProgMan: {0}")]
    ProgMan(#[from] crate::progman::Error),

    #[error("failed to create graphics: {0}")]
    Graphics(#[from] crate::graphics::Error),

    #[error("failed to create renderer: {0}")]
    Renderer(#[from] crate::graphics::SkiaWGLError),

    #[error("an error occurred while calling the win32 API: {0}")]
    WinApi(#[from] WinApiError),

    #[error("Snowland closed the message pipe unexpectedly")]
    PipeClosed(PipeClosedError),
}

#[derive(Debug, Error)]
pub enum PipeClosedError {
    #[error("failed to receive message: {0}")]
    Receive(TryRecvError),

    #[error("failed to send message: {0}")]
    Send(SendError<ControlMessage>),
}
