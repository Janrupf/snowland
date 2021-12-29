mod host;
mod util;

use log::Level;
use snowland_x11_wrapper::{
    GLXError, WindowPropertyChangeMode, XAtom, XDisplay, XDrawable, XLibError, XPixmap, XWindow,
    GLX,
};
use std::time::Duration;
use thiserror::Error;

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

type GLClearColorFn = unsafe extern "C" fn(f32, f32, f32, f32);
type GLClearFn = unsafe extern "C" fn(i32);

fn main_inner() -> Result<(), LinuxHostError> {
    log::debug!("Establishing X11 connection...");
    let display = XDisplay::open()?;

    log::trace!("Display is {:#?}", display);

    let screen = display.default_screen();
    log::info!("Connected to screen {}", screen.number());

    log::debug!("Loading GLX...");
    let glx = GLX::create(&display)?;

    if log::log_enabled!(Level::Debug) {
        let (major, minor) = glx.get_version();

        log::debug!("GLX version is {}.{}", major, minor);
    }

    let root_window = screen.root_window();
    log::debug!("Root window is 0x{:X}", root_window.handle());

    let pixmap = root_window.create_matching_pixmap();

    log::debug!("Pixmap = {:#?}", pixmap);

    delete_root_atoms(&display, &root_window);
    // set_root_atoms(&display, &pixmap, &root_window);

    let window_attributes = root_window.get_attributes();
    let screen = window_attributes.screen();
    let visual = window_attributes.visual();

    let framebuffer_config = glx.find_framebuffer_config(screen, visual)?;
    let context = glx.create_context(screen, &framebuffer_config)?;

    log::trace!("is direct context = {}", context.is_direct());

    display.sync(false);

    let root_window_attributes = root_window.get_attributes();
    log::trace!("root window attributes = {:#?}", root_window_attributes);

    let (glClear, glClearColor) = unsafe {
        let glClear: GLClearFn = std::mem::transmute(glx.lookup_function("glClear").unwrap());
        let glClearColor: GLClearColorFn =
            std::mem::transmute(glx.lookup_function("glClearColor").unwrap());

        (glClear, glClearColor)
    };

    context.make_non_current();
    context.make_current(&root_window);

    display.sync(false);

    loop {
        unsafe {
            glClearColor(0.2, 0.2, 1.0, 1.0);
            glClear(16384);
        }

        context.swap_buffers(&root_window);
        display.sync(false);
    }

    log::trace!("pixmap = {:#?}", pixmap);
    log::debug!("Sleeping for 10 seconds...");
    std::thread::sleep(Duration::from_secs(10));

    Ok(())
}

fn delete_root_atoms(display: &XDisplay, window: &XWindow) {
    let xroot_atom = display.get_or_create_atom("_XROOTPMAP_ID");
    let eroot_atom = display.get_or_create_atom("ESETROOT_PMAP_ID");

    window.delete_property(xroot_atom);
    window.delete_property(eroot_atom);
}

fn set_root_atoms(display: &XDisplay, pixmap: &XPixmap, window: &XWindow) {
    reset_root_atoms(display, window);

    let xroot_atom = display.get_or_create_atom("_XROOTPMAP_ID");
    let eroot_atom = display.get_or_create_atom("ESETROOT_PMAP_ID");

    window.change_property32(
        xroot_atom,
        XAtom::PIXMAP,
        WindowPropertyChangeMode::Replace,
        &[pixmap.drawable_handle() as _],
    );

    window.change_property32(
        eroot_atom,
        XAtom::PIXMAP,
        WindowPropertyChangeMode::Replace,
        &[pixmap.drawable_handle() as _],
    );
}

fn reset_root_atoms(display: &XDisplay, window: &XWindow) -> Option<()> {
    let xroot_atom = display.get_atom("_XROOTPMAP_ID")?;
    let eroot_atom = display.get_atom("ESETROOT_PMAP_ID")?;

    let (xroot_data, _) = window.get_property(xroot_atom, 0, 1, false, XAtom::ANY_PROPERTY_TYPE)?;
    if xroot_data.ty() != XAtom::PIXMAP {
        return None;
    }

    let (eroot_data, _) = window.get_property(eroot_atom, 0, 1, false, XAtom::ANY_PROPERTY_TYPE)?;
    if eroot_data.ty() != XAtom::PIXMAP {
        return None;
    }

    let xroot_pixmap = unsafe { xroot_data.get_as_ref::<u32>() };
    let eroot_pixmap = unsafe { eroot_data.get_as_ref::<u32>() };

    if xroot_pixmap == eroot_pixmap {
        // TODO: This might generate an error which can simply be ignored,
        // figure out how to ignore it!
        // unsafe { xlib_sys::XKillClient(display.handle(), (*xroot_pixmap) as _) };
    }

    Some(())
}

#[derive(Debug, Error)]
enum LinuxHostError {
    #[error(transparent)]
    XLibError(#[from] XLibError),

    #[error(transparent)]
    GLXError(#[from] GLXError),
}
