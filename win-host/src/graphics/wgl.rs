use glium::backend::Backend;
use glium::index::PrimitiveType;
use glium::{Surface as _, SwapBuffersError};
use skia_safe::gpu::gl::FramebufferInfo;
use skia_safe::gpu::{BackendRenderTarget, DirectContext, SurfaceOrigin};
use skia_safe::{Budgeted, Color4f, ColorSpace, ColorType, ImageInfo, Paint, Rect, Surface};
use std::ffi::c_void;
use windows::Win32::Foundation::{HANDLE, HINSTANCE};
use windows::Win32::Graphics::Gdi::{HDC, WGL_SWAP_MAIN_PLANE};
use windows::Win32::Graphics::OpenGL::{
    wglDeleteContext, wglGetCurrentContext, wglGetProcAddress, wglMakeCurrent, wglSwapLayerBuffers,
    GL_RGBA8, HGLRC,
};
use windows::Win32::System::LibraryLoader::{FreeLibrary, GetProcAddress, LoadLibraryA};

struct GliumWGLBackend {
    dc: HDC,
    gl: HGLRC,
    opengl32: HINSTANCE,
}

impl GliumWGLBackend {
    pub unsafe fn new(dc: HDC, gl: HGLRC) -> Self {
        let opengl32 = unsafe { LoadLibraryA("OpenGL32") };

        Self { dc, gl, opengl32 }
    }
}

impl Drop for GliumWGLBackend {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.opengl32) };
    }
}

unsafe impl Backend for GliumWGLBackend {
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        let ok = unsafe { wglSwapLayerBuffers(self.dc, WGL_SWAP_MAIN_PLANE) }.as_bool();

        if ok {
            Ok(())
        } else {
            Err(SwapBuffersError::ContextLost)
        }
    }

    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        let result = wglGetProcAddress(symbol)
            .or_else(|| GetProcAddress(self.opengl32, symbol))
            .map_or(std::ptr::null(), |proc| proc as _);

        log::debug!("GL function {} = {:p}", symbol, result);

        result
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        (400, 400)
    }

    fn is_current(&self) -> bool {
        self.gl == unsafe { wglGetCurrentContext() }
    }

    unsafe fn make_current(&self) {
        wglMakeCurrent(self.dc, self.gl);
    }
}

#[derive(Debug)]
pub struct WGLContext {
    dc: HDC,
    gl: HGLRC,
}

impl WGLContext {
    pub unsafe fn new(dc: HDC, gl: HGLRC) -> Self {
        Self { dc, gl }
    }

    pub fn test_draw(&self) {
        assert!(unsafe { wglMakeCurrent(self.dc, self.gl).as_bool() });

        let wgl_backend = unsafe { GliumWGLBackend::new(self.dc, self.gl) };

        let skia_gl_interface = skia_safe::gpu::gl::Interface::new_load_with(|proc| unsafe {
            wgl_backend.get_proc_address(proc)
        });

        assert!(skia_gl_interface.is_some());
        assert!(skia_gl_interface.as_ref().unwrap().validate());

        let mut context = DirectContext::new_gl(skia_gl_interface, None).unwrap();

        let render_target = BackendRenderTarget::new_gl(
            (400, 400),
            None,
            0,
            FramebufferInfo {
                fboid: 0,
                format: GL_RGBA8,
            },
        );

        let mut surface = Surface::from_backend_render_target(
            &mut context,
            &render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            Some(ColorSpace::new_srgb()),
            None,
        )
        .unwrap();

        let canvas = surface.canvas();

        loop {
            // drawing a frame
            // frame.clear_color(0.0, 0.0, 0.0, 0.0);

            // canvas.clear(Color4f::new(1.0, 1.0, 1.0, 1.0));

            canvas.clear(Color4f::new(1.0, 1.0, 1.0, 1.0));
            canvas.draw_rect(
                Rect::new(0.0, 0.0, 200.0, 200.0),
                &Paint::new(Color4f::new(1.0, 0.0, 0.0, 1.0), None),
            );

            context.flush_and_submit();

            wgl_backend.swap_buffers();
        }
    }
}

impl Drop for WGLContext {
    fn drop(&mut self) {
        unsafe { wglDeleteContext(self.gl) };
    }
}
