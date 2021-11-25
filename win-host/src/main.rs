#![windows_subsystem = "windows"]

use windows::Win32::Graphics::Dwm::DwmFlush;

use snowland_universal::rendering::SnowlandRenderer;
use snowland_universal::Snowland;

use crate::graphics::{Graphics, SkiaWGLSnowlandRender};
use crate::progman::{ProgMan, Worker};
use crate::shell::messenger::{
    HostMessenger, HostToIntegrationMessage, IntegrationToHostMessage, ReceiveResult,
};
use crate::shell::start_shell_integration;
use crate::util::WinApiError;

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

    log::debug!("Creating graphics...");
    let graphics = match Graphics::from_window(worker.get_handle()) {
        Ok(v) => v,
        Err(err) => {
            log::error!("Failed to create graphics: {}", err);
            std::process::exit(1);
        }
    };

    log::debug!("Creating WGL context...");
    let gl = match graphics.create_wgl_context() {
        Ok(v) => v,
        Err(err) => {
            log::error!("Failed to create WGL context: {}", err);
            std::process::exit(1);
        }
    };

    log::debug!("Creating renderer...");
    let renderer = match SkiaWGLSnowlandRender::from_context(gl) {
        Ok(v) => v,
        Err(err) => {
            log::error!("Failed to create renderer: {}", err);
            std::process::exit(1);
        }
    };

    log::debug!(
        "prog_man = {:?}, worker = {:?}, graphics = {:?}",
        prog_man,
        worker,
        graphics,
    );

    let mut messenger = start_shell_integration();

    let mut snowland = Snowland::new(renderer);
    match run_render_loop(&worker, &mut messenger, &mut snowland) {
        Ok(()) => (),
        Err(err) => {
            log::error!("Encountered error while rendering: {0}", err);
            std::process::exit(1);
        }
    };

    std::mem::forget(snowland); // TODO: dropping the renderer hangs it Skia...

    messenger.send(HostToIntegrationMessage::QuitLoop);
    match messenger.join() {
        Err(err) => log::warn!("Join integration thread finished with error: {}", err),
        Ok(()) => (),
    }
}

fn run_render_loop<R>(
    worker: &Worker,
    messenger: &mut HostMessenger,
    snowland: &mut Snowland<R>,
) -> Result<(), Box<dyn std::error::Error>>
where
    R: SnowlandRenderer,
{
    loop {
        match messenger.receive() {
            ReceiveResult::Message(v) => {
                return match v {
                    IntegrationToHostMessage::StopRendering => Ok(()),
                }
            }
            ReceiveResult::Shutdown => return Ok(()),
            ReceiveResult::None => {}
        }

        let (width, height) = worker.get_size()?;
        snowland.resize(width, height)?;
        snowland.render_frame()?;

        unsafe {
            DwmFlush().unwrap();
        }
    }
}
