use std::fmt::Debug;

use egui::epaint::ClippedShape;
use egui_glium::EguiGlium;
use glium::backend::glutin::DisplayCreationError;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::glutin::window::{UserAttentionType, WindowBuilder};
use glium::glutin::ContextBuilder;
use glium::{Display, Surface, SwapBuffersError};
use thiserror::Error;

use crate::ui::panel::EguiPanel;
use crate::util::{Notifier, NotifierImpl};
use crate::ControlMessage;

mod panel;

/// Contains the contents of the user interface.
pub struct SnowlandUI {
    panel: EguiPanel,
    egui: EguiGlium,
    display: Display,
    event_loop: Option<EventLoop<ControlMessage>>,
}

impl SnowlandUI {
    /// Initializes the user interface, but does not display it.
    pub fn new() -> Result<(Self, Notifier<ControlMessage>), Error> {
        let window_builder = WindowBuilder::new()
            .with_resizable(true)
            .with_visible(false)
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

        let egui = EguiGlium::new(&display);
        let notifier = UINotifier::create(event_loop.create_proxy());

        Ok((
            Self {
                panel: EguiPanel::new(),
                display,
                event_loop: Some(event_loop),
                egui,
            },
            notifier,
        ))
    }

    /// Starts the event loop and processes messages.
    pub fn run_loop(&mut self, notifier: &Notifier<ControlMessage>) -> Result<(), Error> {
        let mut run_result = Ok(());
        let run_result_ref = &mut run_result;

        self.event_loop
            .take()
            .unwrap()
            .run_return(
                |event, _, control_flow| match self.run_loop_iteration(event, notifier) {
                    Ok(flow) => *control_flow = flow,
                    Err(err) => {
                        *run_result_ref = Err(err);
                        *control_flow = ControlFlow::Exit;
                    }
                },
            );

        run_result
    }

    /// Runs a single event loop iteration.
    fn run_loop_iteration(
        &mut self,
        event: Event<ControlMessage>,
        notifier: &Notifier<ControlMessage>,
    ) -> Result<ControlFlow, Error> {
        let mut run_frame = || {
            self.run_egui_frame()
                .and_then(|(needs_redraw, shapes)| {
                    self.perform_paint(shapes).map(|()| needs_redraw)
                })
                .map(|redraw| {
                    if redraw {
                        self.display.gl_window().window().request_redraw();
                        ControlFlow::Poll
                    } else {
                        ControlFlow::Wait
                    }
                })
        };

        match event {
            Event::RedrawEventsCleared if cfg!(windows) => run_frame(),
            Event::RedrawRequested(_) if !cfg!(windows) => run_frame(),

            Event::UserEvent(message) => self.process_control_message(message, notifier),

            Event::WindowEvent { event, .. } => {
                if matches!(event, WindowEvent::CloseRequested) {
                    self.display.gl_window().window().set_visible(false);
                    notifier.notify(ControlMessage::CloseUI);
                }

                self.egui.on_event(&event);

                if self.run_egui_frame()?.0 {
                    self.display.gl_window().window().request_redraw();
                    Ok(ControlFlow::Poll)
                } else {
                    Ok(ControlFlow::Wait)
                }
            }

            _ => Ok(ControlFlow::Wait),
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

    /// Runs an egui frame.
    fn run_egui_frame(&mut self) -> Result<(bool, Vec<ClippedShape>), Error> {
        self.egui.begin_frame(&self.display);

        self.panel.run(self.egui.ctx());

        Ok(self.egui.end_frame(&self.display))
    }

    /// Repaints the window using OpenGL.
    fn perform_paint(&mut self, shapes: Vec<ClippedShape>) -> Result<(), Error> {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        self.egui.paint(&self.display, &mut target, shapes);

        Ok(target.finish()?)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to create window: {0}")]
    DisplayCreation(#[from] DisplayCreationError),

    #[error("failed to swap buffers: {0}")]
    SwapBuffers(#[from] SwapBuffersError),
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
