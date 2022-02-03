use skia_safe::gpu::gl::{FramebufferInfo, Interface};
use skia_safe::gpu::{BackendRenderTarget, DirectContext, SurfaceOrigin};
use skia_safe::{ColorType, Surface};
use snowland_core::host::SnowlandRenderer;
use snowland_x11_wrapper::{GLXContext, GLXError, XDrawable, XLibError, XWindow, GLX};
use thiserror::Error;

const GL_RGBA8: u32 = 0x8058;

pub struct SnowlandX11Renderer<'a> {
    skia_context: DirectContext,
    context: GLXContext<'a>,
    window: &'a XWindow<'a>,
}

impl<'a> SnowlandX11Renderer<'a> {
    pub fn init(window: &'a XWindow<'a>) -> Result<Self, Error> {
        let glx = GLX::create(window.display())?;

        let attributes = window.get_attributes();
        let visual = attributes.visual();
        let screen = attributes.screen();

        let framebuffer_config = glx.find_framebuffer_config(screen, visual)?;
        let context = glx.create_context(screen, &framebuffer_config)?;

        context.make_current(window);

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
        })
    }
}

impl<'a> SnowlandRenderer for SnowlandX11Renderer<'a> {
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
            &mut self.skia_context,
            &render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        );

        surface.ok_or(Error::SurfaceCreationFailed)
    }

    fn present(&self) -> Result<(), Self::Error> {
        self.context.swap_buffers(self.window);
        Ok(())
    }

    fn get_size(&self) -> Result<(u64, u64), Self::Error> {
        let geometry = self.window.get_geometry();
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
