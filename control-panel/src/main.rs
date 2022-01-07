mod com;

use crate::com::TestCommunicationChannel;
use nativeshell::codec::Value;
use nativeshell::shell::ContextOptions;
use nativeshell::Context;

nativeshell::include_flutter_plugins!();

fn main() {
    pretty_env_logger::init();
    log::info!("Starting snowland control panel...");

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

    let _test_channel = TestCommunicationChannel::register(&context);

    context.run_loop.borrow().run();

    log::info!("Snowland control panel shutting down!");
}
