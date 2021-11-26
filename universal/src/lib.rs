#![feature(drain_filter)]

use std::time::{Instant, SystemTimeError};

use skia_safe::Surface;
use thiserror::Error;

use crate::control::message_pipe::{message_pipe, MessagePipeEnd};
use crate::control::ControlMessage;
use crate::host::SnowlandHost;
use crate::rendering::SnowlandRenderer;
use crate::scene::{SnowlandScene, XMasCountdown};

pub mod control;
pub mod host;
pub mod rendering;
mod scene;

/// The heart of Snowland, application manager and central controller.
pub struct Snowland<H>
where
    H: SnowlandHost,
{
    scene: Box<dyn SnowlandScene>,
    surface: Option<Surface>,
    last_frame_time: Option<Instant>,
    host: H,
    host_pipe: MessagePipeEnd<ControlMessage>,
}

impl<H> Snowland<H>
where
    H: SnowlandHost,
{
    /// Creates a new snowland by using the given host.
    pub fn new(host: H) -> Result<Self, Error<H>> {
        let (host_pipe, ui_pipe) = message_pipe();

        std::mem::forget(ui_pipe); // TODO: implement the UI side

        Ok(Self {
            host,
            surface: None,
            last_frame_time: None,
            scene: Box::new(XMasCountdown::new()),
            host_pipe,
        })
    }

    /// Starts the snowland run loop.
    pub fn run(mut self) -> Result<(), Error<H>> {
        loop {
            let ui_control_messages = self.collect_ui_control_message();
            let host_control_messages = self
                .host
                .process_messages(&ui_control_messages)
                .map_err(Error::HostError)?;

            if self.process_control_messages(&ui_control_messages)
                || self.process_control_messages(&host_control_messages)
            {
                return Ok(());
            }

            for message in host_control_messages {
                drop(self.host_pipe.send(message)); // TODO: Maybe handle this result?
            }

            let (width, height) = self.host.get_size().map_err(Error::HostError)?;
            self.resize(width, height)?;

            self.render_frame()?;
        }
    }

    fn collect_ui_control_message(&self) -> Vec<ControlMessage> {
        let mut messages = Vec::new();

        while let Ok(v) = self.host_pipe.try_recv() {
            messages.push(v);
        }

        messages
    }

    fn process_control_messages(&mut self, messages: &[ControlMessage]) -> bool {
        messages.contains(&ControlMessage::Exit)
    }

    fn resize(&mut self, width: u64, height: u64) -> Result<(), Error<H>> {
        let needs_surface_recreation = self
            .surface
            .as_ref()
            .map(|s| s.width() as u64 == width && s.height() as u64 == height)
            .unwrap_or(true);

        if needs_surface_recreation {
            let new_surface = self
                .host
                .renderer()
                .create_surface(width, height)
                .map_err(Error::RendererError)?;
            self.surface.replace(new_surface);
        }

        Ok(())
    }

    fn render_frame(&mut self) -> Result<(), Error<H>> {
        let surface = self.surface.as_mut().ok_or(Error::NoSurface)?;

        let width = surface.width();
        let height = surface.height();

        let canvas = surface.canvas();

        let last_frame_time = self
            .last_frame_time
            .replace(Instant::now())
            .unwrap_or_else(Instant::now);

        self.scene.update(
            canvas,
            width as u64,
            height as u64,
            (last_frame_time.elapsed().as_nanos() as f32) / 1000000.0,
        );

        surface.flush_and_submit();

        let renderer = self.host.renderer();
        renderer.present().map_err(Error::RendererError)?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error<H>
where
    H: SnowlandHost,
{
    #[error("no surface to render to")]
    NoSurface,

    #[error(transparent)]
    HostError(H::Error),

    #[error(transparent)]
    RendererError(<<H as SnowlandHost>::Renderer as SnowlandRenderer>::Error),

    #[error("failed to calculate frame time: {0}")]
    TimeError(#[from] SystemTimeError),
}
