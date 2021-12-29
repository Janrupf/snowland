use snowland_universal::control::ControlMessage;
use snowland_universal::util::Notifier;

use crate::cli::CLI;
use crate::graphics::SnowlandX11Renderer;
use snowland_universal::host::{RendererResult, SnowlandHost, SnowlandRendererCreator};
use thiserror::Error;

/// Linux host implementation for snowland.
pub struct LinuxHost {
    cli: CLI,
}

impl LinuxHost {
    pub fn new(
        _notifier: Notifier<ControlMessage>,
        cli: CLI,
    ) -> Result<(Self, Notifier<ControlMessage>), Error> {
        let dummy_notifier = Notifier::from_fn(|_| {});
        Ok((Self { cli }, dummy_notifier))
    }
}

impl SnowlandHost for LinuxHost {
    type Renderer = SnowlandX11Renderer;
    type RendererCreator = LinuxRendererCreator;
    type Error = Error;

    fn prepare_renderer(&mut self) -> Self::RendererCreator {
        LinuxRendererCreator {
            window: self.cli.window,
        }
    }
}

pub struct LinuxRendererCreator {
    window: Option<u64>,
}

impl SnowlandRendererCreator<LinuxHost> for LinuxRendererCreator {
    fn create(self) -> RendererResult<SnowlandX11Renderer, LinuxHost> {
        SnowlandX11Renderer::init(self.window)
    }
}

#[derive(Debug, Error)]
pub enum Error {}
