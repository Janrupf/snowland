mod com;
mod util;

pub use control_panel_macro as mcr;

use crate::com::DartToNativeChannel;
use nativeshell::codec::Value;
use nativeshell::shell::ContextOptions;
use nativeshell::Context;
use snowland_ipc::mio::{Events, Poll};
use snowland_ipc::SnowlandIPC;

nativeshell::include_flutter_plugins!();

fn main() {
    pretty_env_logger::init();
    log::info!("Starting snowland control panel...");

    ipc_test();

    let context = match Context::new(ContextOptions {
        app_namespace: "snowland_control_panel".into(),
        flutter_plugins: flutter_get_plugins(),
        ..Default::default()
    }) {
        Ok(v) => v,
        Err(err) => {
            log::error!("Failed to create flutter context: {}", err);
            std::process::exit(1);
        }
    };

    if let Err(err) = context
        .window_manager
        .borrow_mut()
        .create_window(Value::Null, None)
    {
        log::error!("Failed to create main window: {}", err);
        std::process::exit(1);
    }

    log::debug!("Registering method channels...");
    let _dart_to_native = DartToNativeChannel::register(&context);

    log::debug!("Starting run loop...");
    context.run_loop.borrow().run();

    log::info!("Snowland control panel shutting down!");
}

fn ipc_test() {
    std::thread::spawn(|| {
        ipc_main().unwrap();
    });
}

fn ipc_main() -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Connecting IPC client...");
    let mut ipc = SnowlandIPC::connect_client()?;

    let mut poll = Poll::new()?;
    ipc.register(poll.registry())?;

    let mut events = Events::with_capacity(1024);

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            if ipc.consumes_event(event) {
                let messages = ipc.process_event(event, poll.registry())?;
                log::debug!("Received {} ipc messages: {:#?}", messages.len(), messages);
            }
        }
    }
}
