use std::path::PathBuf;
use std::process::{Command, Stdio};
use thiserror::Error;

#[cfg(target_os = "windows")]
const SNOWLAND_DAEMON_EXE_NAME: &str = "snowland-windows-host.exe";

#[cfg(target_os = "linux")]
const SNOWLAND_DAEMON_EXE_NAME: &str = "snowland-linux-host";

pub fn start_daemon() -> Result<(), DaemonStartError> {
    log::debug!("Attempting to start daemon...");

    let self_path = std::env::current_exe();
    let self_dir = match self_path {
        Ok(p) => p
            .parent()
            .map(|p| p.to_owned())
            .unwrap_or_else(|| PathBuf::from("/")),
        Err(exe_err) => {
            log::warn!(
                "Failed to retrieve path of own executable, falling back to working directory: {}",
                exe_err
            );
            match std::env::current_dir() {
                Ok(p) => p,
                Err(working_dir_err) => {
                    log::warn!(
                        "Failed to retrieve working directory, can't autostart daemon: {}",
                        working_dir_err
                    );
                    return Err(DaemonStartError::FailedToSelfLocate(
                        exe_err,
                        working_dir_err,
                    ));
                }
            }
        }
    };

    let daemon_path = self_dir.join(SNOWLAND_DAEMON_EXE_NAME);
    log::debug!("Looking for snowland daemon at {}", daemon_path.display());
    if !daemon_path.exists() {
        return Err(DaemonStartError::FailedToLocateDaemon(daemon_path));
    }

    // TODO: This might lead to zombie processes on Linux, we should probably set up a
    //       a signal handler for SIGCHLD calling "wait" so that the process is reaped.
    //       However, this would only ever be a problem if the daemon crashes in first place...
    match Command::new(daemon_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(v) => {
            log::debug!("Daemon process spawned as PID {}", v.id());
        }
        Err(err) => {
            log::error!("Failed to start daemon: {}", err);
            return Err(DaemonStartError::Io(err));
        }
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum DaemonStartError {
    #[error("an I/O error occurred: {0}")]
    Io(std::io::Error),

    #[error("could not determine own exe location ({0}) or working directory ({1})")]
    FailedToSelfLocate(std::io::Error, std::io::Error),

    #[error("daemon not found at {}", (.0).display())]
    FailedToLocateDaemon(std::path::PathBuf),
}
