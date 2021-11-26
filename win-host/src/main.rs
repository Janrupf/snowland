#![windows_subsystem = "windows"]

use windows::Win32::Graphics::Dwm::DwmFlush;

use snowland_universal::host::SnowlandHost;
use snowland_universal::{Error, Snowland};

use crate::graphics::{Graphics, SkiaWGLSnowlandRender};
use crate::host::WinHost;
use crate::progman::{ProgMan, Worker};
use crate::shell::messenger::{HostMessenger, HostToIntegrationMessage};
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

    let snowland = Snowland::create_with(WinHost::new).unwrap();
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
