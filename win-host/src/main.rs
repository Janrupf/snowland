#![windows_subsystem = "windows"]

use snowland_core::Snowland;

use crate::graphics::SkiaWGLSnowlandRender;

mod graphics;
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

    let renderer = match SkiaWGLSnowlandRender::init() {
        Ok(v) => v,
        Err(err) => {
            log::error!("Failed to create renderer: {}", err);
            std::process::exit(1);
        }
    };

    let mut snowland = match Snowland::create(renderer) {
        Ok(v) => v,
        Err(err) => {
            log::error!("failed to make it snow: {}", err);
            std::process::exit(1);
        }
    };

    snowland.update_displays(util::display::get_displays());
    if let Err(err) = snowland.load_configuration_from_disk() {
        log::warn!(
            "Failed to load module configuration form disk, starting without modules: {}",
            err
        );
    }

    loop {
        snowland.draw_frame().expect("Failed to draw frame");
        snowland.tick_ipc().expect("Failed to tick IPC");
    }
}
