#![windows_subsystem = "windows"]

use snowland_core::Snowland;

use crate::graphics::{Graphics, SkiaWGLSnowlandRender};
use crate::host::WinHost;
use crate::progman::{ProgMan, Worker};
use crate::shell::start_shell_integration;
use crate::util::WinApiError;

mod graphics;
mod host;
mod progman;
mod shell;
mod util;

fn main() {
    pretty_env_logger::init();
    log::info!(
        "Starting {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let snowland = match Snowland::create_with(WinHost::new) {
        Ok(v) => v,
        Err(err) => {
            log::error!("Failed to make it snow: {}", err);
            std::process::exit(1);
        }
    };

    match snowland.run() {
        Ok(()) => {
            log::debug!("Snowland finished successfully!");
            std::process::exit(0)
        }
        Err(err) => {
            log::error!("Snowland finished with error: {}", err);
            std::process::exit(1)
        }
    }
}
