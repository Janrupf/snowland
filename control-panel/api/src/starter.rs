use snowland_ipc::startup::{StartupShare, WithStartupShare};
use std::process::{Command, Stdio};

pub fn start_new_host() -> Result<StartupShare, std::io::Error> {
    log::trace!("Starting new host process...");

    let share = StartupShare::new()?;
    log::trace!("Startup share for new process: {:?}", share);

    // TODO: Remove hardcoded path
    let child = Command::new("/projects/public/snowland/target/debug/snowland-linux-host")
        .startup_share(&share)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()?;

    log::trace!("Started new host process with pid {}", child.id());

    Ok(share)
}
