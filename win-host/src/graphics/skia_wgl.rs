use skia_safe::gpu::gl::FramebufferInfo;
use skia_safe::gpu::{BackendRenderTarget, DirectContext, SurfaceOrigin};
use skia_safe::*;
use thiserror::Error;
use windows::Win32::Graphics::OpenGL::GL_RGBA8;

use snowland_universal::rendering::SnowlandRenderer;

use crate::graphics::WGLContext;

/// Snowland renderer based on WGL.
#[derive(Debug)]
pub struct SkiaWGLSnowlandRender {
    skia_context: DirectContext,
    wgl_context: WGLContext,
}

impl SkiaWGLSnowlandRender {
    /// Creates a Snowland renderer from a WGL context.
    pub fn from_context(wgl_context: WGLContext) -> Result<Self, Error> {
        wgl_context
            .make_current()
            .map_err(Error::MakeContextCurrent)?;

        let skia_interface =
            gpu::gl::Interface::new_load_with(|proc| wgl_context.lookup_wgl_proc(proc))
                .ok_or(Error::InterfaceCreationFailed)?;

        let skia_context = DirectContext::new_gl(Some(skia_interface), None)
            .ok_or(Error::ContextCreationFailed)?;

        Ok(Self {
            wgl_context,
            skia_context,
        })
    }
}

impl SnowlandRenderer for SkiaWGLSnowlandRender {
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
            BackendRenderTarget::new_gl((width as i32, height as i32), None, 0, framebuffer);

        let surface = Surface::from_backend_render_target(
            &mut self.skia_context,
            &render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None,
        );

        surface
            .ok_or(Error::SurfaceCreationFailed)
            .map_err(Into::into)
    }

    fn present(&self) -> Result<(), Self::Error> {
        self.wgl_context
            .swap_buffers()
            .map_err(Error::SwapBuffers)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to make context current: {0}")]
    MakeContextCurrent(crate::graphics::wgl::Error),

    #[error("failed to swap buffers: {0}")]
    SwapBuffers(crate::graphics::wgl::Error),

    #[error("failed to create skia interface")]
    InterfaceCreationFailed,

    #[error("failed to create GPU context")]
    ContextCreationFailed,

    #[error("{0}x{1} is bigger than the supported size")]
    SizeOutOfBounds(u64, u64),

    #[error("failed to create surface")]
    SurfaceCreationFailed,
}
