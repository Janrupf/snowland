use crate::cli::Cli;
use crate::graphics::SnowlandX11Renderer;
use snowland_universal::rendering::display::Display;
use snowland_universal::Snowland;
use snowland_x11_wrapper::{XDisplay, XScreen, XWindow};

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

    let display = match XDisplay::open(None) {
        Ok(v) => v,
        Err(err) => {
            log::error!("{}", err);
            std::process::exit(1);
        }
    };

    let screen = display.default_screen();
    let (window, _is_fullscreen) = get_render_target(&screen, &cli);

    let renderer = match SnowlandX11Renderer::init(&window) {
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

    snowland.update_displays(get_displays(&screen));
    snowland
        .load_configuration_from_disk()
        .expect("Failed to load modules from disk");

    loop {
        snowland.draw_frame().expect("Failed to draw frame");
    }
}

/// Retrieves all connected video outputs of the X screen.
fn get_displays(screen: &XScreen) -> Vec<Display> {
    screen
        .get_monitors()
        .into_iter()
        .enumerate()
        .map(|(i, monitor)| {
            let fake_data = format!("Monitor {}", i);

            let name = format!(
                "{}: {}",
                i,
                monitor.monitor_name.as_ref().unwrap_or(&fake_data)
            );
            let serial = monitor
                .monitor_serial
                .map(|i| i.to_string())
                .unwrap_or(fake_data);

            Display::new(
                name,
                serial,
                monitor.primary,
                monitor.x,
                monitor.y,
                monitor.width,
                monitor.height,
            )
        })
        .collect()
}

/// Retrieves the window the renderer should draw to.
fn get_render_target<'a>(screen: &'a XScreen<'a>, cli: &Cli) -> (XWindow<'a>, bool) {
    match cli.window {
        None => (screen.root_window(), true),
        Some(i) => (unsafe { XWindow::new(i as _, screen.display()) }, true),
    }
}
