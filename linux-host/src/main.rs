use crate::cli::Cli;
use crate::graphics::SnowlandX11Renderer;
use snowland_universal::rendering::display::Display;
use snowland_universal::Snowland;
use snowland_x11_wrapper::{XDisplay, XScreen, XWindow};
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, Ordering};

mod cli;
mod graphics;

static SHOULD_SHUTDOWN: AtomicBool = AtomicBool::new(false);

fn main() {
    pretty_env_logger::init();

    let cli = cli::parse();
    log::trace!("cli = {:#?}", cli);

    log::info!(
        "Starting {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    setup_signal_handler();

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

    while !SHOULD_SHUTDOWN.load(Ordering::Relaxed) {
        snowland.draw_frame().expect("Failed to draw frame");
        snowland.tick_ipc().expect("Failed to tick IPC");
    }

    log::info!("Shutting down snowland...");
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

unsafe extern "system" fn sigint_handler(
    _signal: libc::c_int,
    _info: *mut libc::siginfo_t,
    _data: *mut libc::c_void,
) {
    SHOULD_SHUTDOWN.store(true, Ordering::Relaxed);
}

fn setup_signal_handler() {
    unsafe {
        let mut action: libc::sigaction = MaybeUninit::zeroed().assume_init();
        // This assigns sa_handler, which is not there in the libc crate...
        action.sa_sigaction = sigint_handler as _;
        libc::sigemptyset(&mut action.sa_mask);
        action.sa_flags = 0;

        libc::sigaction(libc::SIGINT, &action, std::ptr::null_mut());
    }
}
