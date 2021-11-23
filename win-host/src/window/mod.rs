mod renderer;
mod snowflake;

use crate::window::renderer::SnowlandRenderer;
use skulpin::rafx::api::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use skulpin::rafx::api::{RafxError, RafxExtents2D};
use skulpin::{skia_safe, CoordinateSystem, LogicalSize, Renderer, RendererBuilder};
use thiserror::Error;
use windows::Win32::Foundation::HWND;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Fullscreen, Window};

pub struct RenderWindow {
    events: EventLoop<()>,
    window: Window,
    renderer: Renderer,
    internal_renderer: SnowlandRenderer,
}

impl RenderWindow {
    pub fn new() -> Result<Self, Error> {
        let events = winit::event_loop::EventLoop::<()>::with_user_event();
        let window = winit::window::WindowBuilder::new()
            .with_title("Snowland")
            .with_transparent(true)
            .with_decorations(false)
            .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .build(&events)?;

        let window_size = window.inner_size();
        let window_extents = RafxExtents2D {
            width: window_size.width,
            height: window_size.height,
        };

        let renderer = RendererBuilder::new()
            .coordinate_system(CoordinateSystem::Physical)
            .build(&window, window_extents)?;

        Ok(RenderWindow {
            events,
            window,
            renderer,
            internal_renderer: SnowlandRenderer::new(),
        })
    }

    pub fn get_window_handle(&self) -> HWND {
        if let RawWindowHandle::Windows(handle) = self.window.raw_window_handle() {
            HWND(handle.hwnd as isize)
        } else {
            unreachable!("Win32 window did not have a HWND")
        }
    }

    pub fn run(&mut self) {
        self.events
            .run_return(|event, _window_target, control_flow| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                Event::MainEventsCleared => self.window.request_redraw(),
                Event::RedrawRequested(_) => {
                    let window_size = self.window.inner_size();
                    let window_extents = RafxExtents2D {
                        width: window_size.width,
                        height: window_size.height,
                    };

                    if let Err(err) = self.renderer.draw(
                        window_extents,
                        self.window.scale_factor(),
                        |canvas, coordinate_system_helper| {
                            self.internal_renderer
                                .draw_frame(canvas, coordinate_system_helper);
                        },
                    ) {
                        log::error!("Failed to render frame: {}", err);
                        *control_flow = ControlFlow::Exit;
                    }
                }
                _ => {}
            });
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to create the window: {0}")]
    WindowCreation(#[from] winit::error::OsError),

    #[error("failed to create renderer: {0}")]
    RendererCreation(#[from] RafxError),
}
