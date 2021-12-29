use crate::host::LinuxHost;
use snowland_universal::{Error, Snowland};

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

    let snowland = match Snowland::create_with(|notifier| LinuxHost::new(notifier, cli)) {
        Ok(v) => v,
        Err(err) => {
            log::error!("Failed to make it snow: {}", err);
            std::process::exit(1)
        }
    };

    match snowland.run() {
        Ok(()) => {
            log::debug!("Snowland finished successfully!");
            std::process::exit(0);
        }
        Err(err) => {
            log::error!("Snowland finished with error: {}", err);
            std::process::exit(1);
        }
    }
}
