mod graphics;
mod host;
mod util;

use log::Level;
use thiserror::Error;

use crate::graphics::{GLXError, XDisplay, XLibError, GLX};

fn main() {
    pretty_env_logger::init();
    log::info!(
        "Starting {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    if let Err(err) = main_inner() {
        log::error!("Failed to run host: {}", err);
        std::process::exit(1);
    }

    log::info!("Snowland linux host exited successfully!");
}

type GLClearColorFn = unsafe extern "C" fn(f32, f32, f32, f32);
type GLClearFn = unsafe extern "C" fn(i32);

fn main_inner() -> Result<(), LinuxHostError> {
    log::debug!("Establishing X11 connection...");
    let display = XDisplay::open()?;

    log::trace!("Display is {:#?}", display);

    let screen = display.default_screen();
    log::info!("Connected to screen {}", screen.number());

    log::debug!("Loading GLX...");
    let glx = GLX::create(&display)?;

    if log::log_enabled!(Level::Debug) {
        let (major, minor) = glx.get_version();

        log::debug!("GLX version is {}.{}", major, minor);
    }

    let root_window = screen.root_window();
    log::debug!("Root window is 0x{:X}", root_window.handle());

    let context = glx.create_context(&root_window)?;
    log::trace!("is direct context = {}", context.is_direct());

    display.sync(false);

    let root_window_attributes = root_window.get_attributes();
    log::trace!("root window attributes = {:#?}", root_window_attributes);

    let (glClear, glClearColor) = unsafe {
        let glClear: GLClearFn = std::mem::transmute(glx.lookup_function("glClear").unwrap());
        let glClearColor: GLClearColorFn =
            std::mem::transmute(glx.lookup_function("glClearColor").unwrap());

        (glClear, glClearColor)
    };

    context.make_non_current();
    context.make_current(&root_window);

    unsafe {
        glClearColor(0.2, 0.2, 1.0, 1.0);
        glClear(16384);
    }

    context.swap_buffers(&root_window);

    Ok(())
}

#[derive(Debug, Error)]
enum LinuxHostError {
    #[error(transparent)]
    XLibError(#[from] XLibError),

    #[error(transparent)]
    GLXError(#[from] GLXError),
}
