use crate::startup::StartupShareError;
use std::env::VarError;
use std::io::Read;
use std::ops::Deref;
use std::os::unix::io::RawFd;
use std::process::Command;

#[cfg(feature = "poll")]
use mio::{Interest, Registry, Token};

#[cfg(feature = "poll")]
use mio::unix::SourceFd;

#[derive(Debug)]
#[repr(transparent)]
struct CloseFdOnDrop {
    inner: RawFd,
}

impl CloseFdOnDrop {
    unsafe fn new(inner: RawFd) -> Self {
        Self { inner }
    }

    fn inner(&self) -> RawFd {
        self.inner
    }
}

impl Drop for CloseFdOnDrop {
    fn drop(&mut self) {
        if self.inner != -1 {
            unsafe { libc::close(self.inner) };
        }
    }
}

impl Deref for CloseFdOnDrop {
    type Target = RawFd;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug)]
pub(super) struct StartupShareInner {
    read_fd: CloseFdOnDrop,
    write_fd: CloseFdOnDrop,
}

const STARTUP_SHARE_ENVIRONMENT_VAR: &str = "SNOWLAND_STARTUP_SHARE";

impl StartupShareInner {
    pub fn new() -> Result<Self, StartupShareError> {
        let mut descriptors = [0; 2];
        let result = unsafe { libc::pipe(descriptors.as_mut_ptr()) };

        if result == -1 {
            Err(StartupShareError::Io(std::io::Error::last_os_error()))
        } else {
            unsafe {
                #[cfg(feature = "poll")]
                Self::make_nonblocking(descriptors[0])?;

                Ok(Self {
                    read_fd: CloseFdOnDrop::new(descriptors[0]),
                    write_fd: CloseFdOnDrop::new(descriptors[1]),
                })
            }
        }
    }

    #[cfg(feature = "poll")]
    unsafe fn make_nonblocking(fd: RawFd) -> Result<(), StartupShareError> {
        let flags = libc::fcntl(fd, libc::F_GETFL);

        if flags == -1 {
            return Err(StartupShareError::Io(std::io::Error::last_os_error()));
        }

        let result = libc::fcntl(fd, libc::F_SETFL, flags & libc::O_NONBLOCK);
        if result == -1 {
            Err(StartupShareError::Io(std::io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    pub fn apply_to(&self, command: &mut Command) {
        command.env(
            STARTUP_SHARE_ENVIRONMENT_VAR,
            format!("{}", self.write_fd.inner()),
        );
    }

    pub fn read(&self, buffer: &mut [u8]) -> Result<usize, std::io::Error> {
        let read =
            unsafe { libc::read(*self.read_fd, buffer.as_mut_ptr() as _, buffer.len() as _) };

        if read == -1 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(read as _)
    }

    pub fn fulfill_current(data: &[u8]) -> Result<(), StartupShareError> {
        if data.is_empty() {
            return Err(StartupShareError::BadDataSize(0));
        }

        match std::env::var(STARTUP_SHARE_ENVIRONMENT_VAR) {
            Ok(v) => {
                std::env::remove_var(STARTUP_SHARE_ENVIRONMENT_VAR);

                let fd = match v.parse::<RawFd>() {
                    Ok(v) => v,
                    Err(err) => {
                        log::error!(
                            "Startup share descriptor value {} is not a valid i32: {}",
                            v,
                            err
                        );

                        return Err(StartupShareError::Invalid);
                    }
                };

                let fd = unsafe { CloseFdOnDrop::new(fd) };

                let mut buffer = data;

                while !buffer.is_empty() {
                    let written = unsafe {
                        libc::write(
                            *fd,
                            buffer.as_ptr() as *const libc::c_void,
                            buffer.len() as _,
                        )
                    };

                    if written == -1 {
                        return Err(StartupShareError::Io(std::io::Error::last_os_error()));
                    }

                    buffer = &buffer[(written as usize)..];
                }

                Ok(())
            }
            Err(VarError::NotUnicode(_)) => {
                log::error!("Startup share descriptor value was not valid unicode");

                Err(StartupShareError::Invalid)
            }
            Err(VarError::NotPresent) => Err(StartupShareError::NotPresent),
        }
    }
}

#[cfg(feature = "poll")]
impl mio::event::Source for StartupShareInner {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> std::io::Result<()> {
        if interests.is_writable() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                Box::new(StartupShareError::ReadOnly),
            ));
        }

        SourceFd(&self.read_fd).register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> std::io::Result<()> {
        if interests.is_writable() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                Box::new(StartupShareError::ReadOnly),
            ));
        }

        SourceFd(&self.read_fd).reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> std::io::Result<()> {
        SourceFd(&self.read_fd).deregister(registry)
    }
}

impl Read for StartupShareInner {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        StartupShareInner::read(self, buf)
    }
}
