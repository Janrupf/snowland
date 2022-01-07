use crate::graphics::SnowlandX11Renderer;
use snowland_universal::Snowland;

mod cli;
mod graphics;
mod host;

fn main() {
    pretty_env_logger::init();

    let cli = cli::parse();
    log::trace!("cli = {:#?}", cli);

    log::info!(
        "Starting {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let renderer = match SnowlandX11Renderer::init(cli.window) {
        Ok(v) => v,
        Err(err) => {
            log::error!("Failed to create renderer: {}", err);
            std::process::exit(1);
        }
    };

    let mut snowland = match Snowland::create(renderer) {
        Ok(v) => v,
        Err(err) => {
            log::error!("Failed to make it snow: {}", err);
            std::process::exit(1)
        }
    };

    loop {
        snowland.draw_frame().expect("Failed to draw frame");
    }
}
