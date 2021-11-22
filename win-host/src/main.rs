use crate::progman::ProgMan;
use crate::util::WinApiError;

mod progman;
mod util;

fn main() {
    pretty_env_logger::init();
    log::info!(
        "Starting {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    log::debug!("Creating ProgMan...");
    let prog_man = match ProgMan::new() {
        Ok(v) => v,
        Err(err) => {
            log::error!("Failed to create ProgMan instance: {}", err);
            std::process::exit(1);
        }
    };

    log::debug!("Creating Worker...");
    let worker = match prog_man.get_or_create_worker() {
        Ok(v) => v,
        Err(err) => {
            log::error!("Failed to create worker: {}", err);
            std::process::exit(1)
        }
    };

    log::debug!("prog_man = {:?}, worker = {:?}", prog_man, worker);
}
