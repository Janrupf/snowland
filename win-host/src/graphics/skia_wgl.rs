use skia_safe::gpu::gl::FramebufferInfo;
use skia_safe::gpu::{BackendRenderTarget, DirectContext, SurfaceOrigin};
use skia_safe::*;
use thiserror::Error;
use windows::Win32::Graphics::Dwm::DwmFlush;
use windows::Win32::Graphics::OpenGL::GL_RGBA8;

use snowland_core::host::SnowlandRenderer;

use crate::graphics::WGLContext;
use crate::{Graphics, ProgMan, WinApiError, Worker};

/// Snowland renderer based on WGL.
#[derive(Debug)]
pub struct SkiaWGLSnowlandRender {
    skia_context: DirectContext,
    wgl_context: WGLContext,
    _graphics: Graphics,
    worker: Worker,
    _prog_man: ProgMan,
}

impl SkiaWGLSnowlandRender {
    pub fn init() -> Result<Self, Error> {
        let prog_man = ProgMan::new()?;
        let worker = prog_man.get_or_create_worker()?;
        let graphics = Graphics::from_window(worker.get_handle())?;
        let wgl = graphics.create_wgl_context()?;

        Self::from_context(graphics, worker, prog_man, wgl)
    }

    /// Creates a Snowland renderer from a WGL context.
    pub fn from_context(
        graphics: Graphics,
        worker: Worker,
        prog_man: ProgMan,
        wgl_context: WGLContext,
    ) -> Result<Self, Error> {
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
            _graphics: graphics,
            worker,
            _prog_man: prog_man,
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

        unsafe { DwmFlush() }?;

        Ok(())
    }

    fn get_size(&self) -> Result<(u64, u64), Self::Error> {
        Ok(self.worker.get_size()?)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to create ProgMan: {0}")]
    ProgMan(#[from] crate::progman::Error),

    #[error("failed to create graphics: {0}")]
    Graphics(#[from] crate::graphics::Error),

    #[error("an error occurred while calling the win32 API: {0}")]
    WinApi(#[from] WinApiError),

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
