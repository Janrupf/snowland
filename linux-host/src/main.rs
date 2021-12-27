mod graphics;
mod util;

use log::Level;
use thiserror::Error;

use crate::graphics::{GLXError, XLibDisplay, XLibError, GLX};

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

fn main_inner() -> Result<(), LinuxHostError> {
    log::debug!("Establishing X11 connection...");
    let display = XLibDisplay::open()?;

    log::trace!("Display is {:#?}", display);

    log::debug!("Loading GLX...");
    let glx = GLX::create(&display)?;

    if log::log_enabled!(Level::Debug) {
        let (major, minor) = glx.get_version();

        log::debug!("GLX version is {}.{}", major, minor);
    }

    let framebuffer_configs = glx.choose_framebuffer_config()?;
    log::trace!("framebuffer_configs = {:#?}", framebuffer_configs);

    Ok(())
}

#[derive(Debug, Error)]
enum LinuxHostError {
    #[error(transparent)]
    XLibError(#[from] XLibError),

    #[error(transparent)]
    GLXError(#[from] GLXError),
}
