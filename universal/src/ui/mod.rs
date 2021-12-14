use std::fmt::Debug;
use std::time::Instant;

use glium::backend::glutin::DisplayCreationError;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::error::ExternalError;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::glutin::window::{UserAttentionType, WindowBuilder};
use glium::glutin::ContextBuilder;
use glium::{Display, Surface, SwapBuffersError};
use imgui::{Context, FontConfig, FontSource};
use imgui_glium_renderer::{Renderer, RendererError};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use thiserror::Error;

use crate::io::ConfigIO;
use crate::rendering::fonts::{get_embedded_font_bytes, Font};
use crate::ui::panel::MainPanel;
use crate::util::{Notifier, NotifierImpl};
use crate::{ControlMessage, RendererController};

mod module_list;
mod panel;

/// The window is initially visible in debug builds to ease programming.
const WINDOW_INITIALLY_VISIBLE: bool = cfg!(debug_assertions);

/// Contains the contents of the user interface.
pub struct SnowlandUI {
    panel: MainPanel,
    platform: WinitPlatform,
    imgui: Context,
    renderer: Renderer,
    display: Display,
    event_loop: Option<EventLoop<ControlMessage>>,
    last_frame: Instant,
    is_visible: bool,
}

impl SnowlandUI {
    /// Initializes the user interface, but does not display it.
    pub fn new() -> Result<(Self, Notifier<ControlMessage>), Error> {
        let window_builder = WindowBuilder::new()
            .with_resizable(true)
            .with_visible(WINDOW_INITIALLY_VISIBLE)
            .with_inner_size(LogicalSize {
                width: 640u32,
                height: 420u32,
            })
            .with_title("Snowland Control Panel");

        let context_builder = ContextBuilder::new()
            .with_depth_buffer(0)
            .with_srgb(true)
            .with_stencil_buffer(0)
            .with_vsync(true);

        let event_loop = EventLoop::<ControlMessage>::with_user_event();
        let display = Display::new(window_builder, context_builder, &event_loop)?;

        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        {
            let gl_window = display.gl_window();
            let window = gl_window.window();
            platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);
        }

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::TtfData {
            data: get_embedded_font_bytes(Font::RobotoRegular),
            size_pixels: font_size,
            config: Some(FontConfig {
                size_pixels: font_size,
                rasterizer_multiply: 1.75,
                ..Default::default()
            }),
        }]);

        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        let renderer = Renderer::init(&mut imgui, &display)?;

        let notifier = UINotifier::create(event_loop.create_proxy());

        Ok((
            Self {
                panel: MainPanel::new(),
                platform,
                imgui,
                renderer,
                display,
                event_loop: Some(event_loop),
                last_frame: Instant::now(),
                is_visible: WINDOW_INITIALLY_VISIBLE,
            },
            notifier,
        ))
    }

    /// Starts the event loop and processes messages.
    pub fn run_loop(
        &mut self,
        notifier: &Notifier<ControlMessage>,
        controller: &RendererController,
    ) -> Result<(), Error> {
        let mut run_result = Ok(());
        let run_result_ref = &mut run_result;

        self.event_loop
            .take()
            .unwrap()
            .run_return(|event, _, control_flow| {
                match self.run_loop_iteration(event, notifier, controller) {
                    Ok(flow) => *control_flow = flow,
                    Err(err) => {
                        *run_result_ref = Err(err);
                        *control_flow = ControlFlow::Exit;
                    }
                }
            });

        run_result
    }

    /// Runs a single event loop iteration.
    fn run_loop_iteration(
        &mut self,
        event: Event<ControlMessage>,
        notifier: &Notifier<ControlMessage>,
        controller: &RendererController,
    ) -> Result<ControlFlow, Error> {
        let mut run_frame = || {
            let ui = self.imgui.frame();

            self.panel.run(&ui, controller);

            let gl_window = self.display.gl_window();

            let mut target = self.display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            self.platform.prepare_render(&ui, gl_window.window());

            let draw_data = ui.render();
            self.renderer.render(&mut target, draw_data)?;

            target.finish()?;

            Ok(ControlFlow::Poll)
        };

        match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                self.imgui
                    .io_mut()
                    .update_delta_time(now - std::mem::replace(&mut self.last_frame, now));

                Ok(ControlFlow::Poll)
            }

            Event::MainEventsCleared => {
                let gl_window = self.display.gl_window();
                self.platform
                    .prepare_frame(self.imgui.io_mut(), gl_window.window())?;

                gl_window.window().request_redraw();
                Ok(ControlFlow::Wait)
            }

            Event::RedrawRequested(_) if self.is_visible => run_frame(),
            // Event::RedrawEventsCleared if cfg!(windows) => run_frame(),
            // Event::RedrawRequested(_) if !cfg!(windows) => run_frame(),
            Event::UserEvent(message) => self.process_control_message(message, notifier),

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                self.display.gl_window().window().set_visible(false);
                self.is_visible = false;

                let modules = self.panel.get_modules();
                log::info!("Saving config file because window has been closed...");
                if let Err(err) = ConfigIO::save(modules) {
                    log::error!("Failed to save config file: {}", err);
                } else {
                    log::info!("Config file saved successfully!");
                }

                notifier.notify(ControlMessage::CloseUI);

                Ok(ControlFlow::Wait)
            }

            Event::RedrawEventsCleared => Ok(if self.is_visible {
                ControlFlow::Poll
            } else {
                ControlFlow::Wait
            }),

            event if self.is_visible => {
                let gl_window = self.display.gl_window();
                self.platform
                    .handle_event(self.imgui.io_mut(), gl_window.window(), &event);

                Ok(ControlFlow::Poll)
            }

            _ if !self.is_visible => Ok(ControlFlow::Wait),
            _ => unreachable!("Unhandled event even though all possibilities are covered"),
        }
    }

    /// Processes a control message.
    fn process_control_message(
        &mut self,
        message: ControlMessage,
        notifier: &Notifier<ControlMessage>,
    ) -> Result<ControlFlow, Error> {
        match message {
            ControlMessage::OpenUI => {
                self.display.gl_window().window().set_visible(true);
                self.is_visible = true;

                self.display
                    .gl_window()
                    .window()
                    .request_user_attention(Some(UserAttentionType::Informational));
            }
            ControlMessage::Exit => {
                notifier.notify(ControlMessage::Exit);
                return Ok(ControlFlow::Exit);
            }
            _ => {}
        }

        Ok(ControlFlow::Wait)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to create window: {0}")]
    DisplayCreation(#[from] DisplayCreationError),

    #[error("imgui renderer failed: {0}")]
    Renderer(#[from] RendererError),

    #[error("failed to swap buffers: {0}")]
    SwapBuffers(#[from] SwapBuffersError),

    #[error("an external error occurred: {0}")]
    External(#[from] ExternalError),
}

/// Implementation of a notifier for the UI based on a message loop.
struct UINotifier<T>
where
    T: Send + 'static,
{
    inner: EventLoopProxy<T>,
}

impl<T> UINotifier<T>
where
    T: Send,
{
    /// Creates a new notifier for the given loop proxy.
    fn create(inner: EventLoopProxy<T>) -> Notifier<T> {
        Notifier::from_impl(Self { inner })
    }
}

impl<T> NotifierImpl<T> for UINotifier<T>
where
    T: Send,
{
    fn notify(&self, value: T) {
        drop(self.inner.send_event(value))
    }

    fn replicate(&self) -> Notifier<T> {
        UINotifier::create(self.inner.clone())
    }
}
