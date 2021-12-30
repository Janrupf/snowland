use skia_safe::gpu::gl::{FramebufferInfo, Interface};
use skia_safe::gpu::{BackendRenderTarget, DirectContext, SurfaceOrigin};
use skia_safe::{ColorType, Surface};
use snowland_universal::control::ControlMessage;
use snowland_universal::host::SnowlandRenderer;
use snowland_universal::rendering::display::Display;
use snowland_universal::util::Notifier;
use snowland_x11_wrapper::{GLXContext, GLXError, XDisplay, XDrawable, XLibError, XWindow, GLX};
use std::pin::Pin;
use thiserror::Error;

const GL_RGBA8: u32 = 0x8058;

struct Inner {
    skia_context: DirectContext,
    context: GLXContext<'static>,
    window: XWindow<'static>,
    display: Pin<Box<XDisplay>>,
}

impl Inner {
    pub fn init(notifier: Notifier<ControlMessage>, window: Option<u64>) -> Result<Self, Error> {
        let display = Box::pin(XDisplay::open(None)?);

        // TODO: This entire lifetime extension is more than hacky, find a better way!
        let display_static: &'static XDisplay =
            unsafe { &*(display.as_ref().get_ref() as *const _) };
        let glx = GLX::create(display_static)?;

        let window = match window {
            None => display_static.default_screen().root_window(),
            // TODO: This is untrusted user input and WILL crash if invalid
            Some(id) => unsafe { XWindow::new(id, display_static) },
        };

        let attributes = window.get_attributes();
        let visual = attributes.visual();
        let screen = attributes.screen();

        let displays = screen
            .get_monitors()
            .into_iter()
            .enumerate()
            .map(|(i, monitor)| {
                let fake_data = format!("Monitor {}", i);

                let name = format!(
                    "{}: {}",
                    i,
                    monitor.monitor_name.as_ref().unwrap_or(&fake_data)
                );
                let serial = monitor
                    .monitor_serial
                    .map(|i| i.to_string())
                    .unwrap_or(fake_data);

                Display::new(
                    name,
                    serial,
                    monitor.primary,
                    monitor.x,
                    monitor.y,
                    monitor.width,
                    monitor.height,
                )
            })
            .collect();

        notifier.notify(ControlMessage::UpdateDisplays(displays));

        let framebuffer_config = glx.find_framebuffer_config(screen, visual)?;
        let context = glx.create_context(screen, &framebuffer_config)?;

        context.make_current(&window);

        let skia_interface = Interface::new_load_with(|proc| {
            if proc.starts_with("egl") {
                return std::ptr::null();
            }

            let result = match glx.lookup_function(proc) {
                None => std::ptr::null(),
                Some(v) => v as _,
            };

            log::debug!("Looking up OpenGL function {} = {:p}", proc, result);

            result
        })
        .ok_or(Error::InterfaceCreationFailed)?;

        let skia_context = DirectContext::new_gl(Some(skia_interface), None)
            .ok_or(Error::ContextCreationFailed)?;

        Ok(Self {
            skia_context,
            context,
            window,
            display,
        })
    }
}

pub struct SnowlandX11Renderer {
    inner: Inner,
}

impl SnowlandX11Renderer {
    pub fn init(notifier: Notifier<ControlMessage>, window: Option<u64>) -> Result<Self, Error> {
        Inner::init(notifier, window).map(|inner| Self { inner })
    }
}

impl SnowlandRenderer for SnowlandX11Renderer {
    type Error = Error;

    fn create_surface(&mut self, width: u64, height: u64) -> Result<Surface, Self::Error> {
        if width > i32::MAX as u64 || height > i32::MAX as u64 {
            return Err(Error::SizeOutOfBounds(width, height));
        }

        let framebuffer = FramebufferInfo {
            fboid: 0,
            format: GL_RGBA8,
        };

        let render_target =
            BackendRenderTarget::new_gl((width as _, height as _), None, 0, framebuffer);

        let surface = Surface::from_backend_render_target(
            &mut self.inner.skia_context,
            &render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        );

        surface.ok_or(Error::SurfaceCreationFailed)
    }

    fn present(&self) -> Result<(), Self::Error> {
        self.inner.context.swap_buffers(&self.inner.window);
        self.inner.display.sync(true);
        Ok(())
    }

    fn get_size(&self) -> Result<(u64, u64), Self::Error> {
        let geometry = self.inner.window.get_geometry();
        Ok((geometry.width as _, geometry.height as _))
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("an error occurred while communicating with the X server: {0}")]
    X(#[from] XLibError),

    #[error("an error occurred while interacting with GLX: {0}")]
    Glx(#[from] GLXError),

    #[error("{0}x{1} is bigger than the supported size")]
    SizeOutOfBounds(u64, u64),

    #[error("failed to create skia interface")]
    InterfaceCreationFailed,

    #[error("failed to create GPU context")]
    ContextCreationFailed,

    #[error("failed to create surface")]
    SurfaceCreationFailed,
}
