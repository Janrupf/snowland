use std::io::{IoSliceMut, Read};
use std::process::Command;
use thiserror::Error;

#[cfg(feature = "poll")]
use mio::event::Source;

#[cfg(feature = "poll")]
use mio::{Interest, Registry, Token};

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod os_dependent;

#[derive(Debug)]
pub struct StartupShare {
    inner: os_dependent::StartupShareInner,
}

impl StartupShare {
    pub fn new() -> Result<Self, StartupShareError> {
        let inner = os_dependent::StartupShareInner::new()?;
        Ok(Self { inner })
    }

    pub fn apply_to(&self, command: &mut Command) {
        self.inner.apply_to(command);
    }

    pub fn fulfill_current(data: &[u8]) -> Result<(), StartupShareError> {
        os_dependent::StartupShareInner::fulfill_current(data)
    }
}

impl Read for StartupShare {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.inner.read_vectored(bufs)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.inner.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.inner.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.inner.read_exact(buf)
    }
}

#[cfg(feature = "poll")]
impl Source for StartupShare {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> std::io::Result<()> {
        self.inner.register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> std::io::Result<()> {
        self.inner.reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> std::io::Result<()> {
        self.inner.deregister(registry)
    }
}

#[derive(Debug, Error)]
pub enum StartupShareError {
    #[error("an io error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("no startup share was found")]
    NotPresent,

    #[error("a startup share was found, but its value was invalid")]
    Invalid,

    #[error("data of the size {0} can't be stored in a startup share")]
    BadDataSize(usize),

    #[error("the startup share can only be read")]
    ReadOnly,
}

impl StartupShareError {
    pub fn into_io_error(self) -> std::io::Error {
        if let Self::Io(err) = self {
            return err;
        }

        match &self {
            StartupShareError::Io(_) => unreachable!(),
            StartupShareError::NotPresent => {
                std::io::Error::new(std::io::ErrorKind::NotFound, Box::new(self))
            }
            StartupShareError::Invalid => {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, Box::new(self))
            }
            StartupShareError::BadDataSize(_) => {
                std::io::Error::new(std::io::ErrorKind::InvalidData, Box::new(self))
            }
            StartupShareError::ReadOnly => {
                std::io::Error::new(std::io::ErrorKind::Unsupported, Box::new(self))
            }
        }
    }
}

impl From<StartupShareError> for std::io::Error {
    fn from(err: StartupShareError) -> Self {
        err.into_io_error()
    }
}

pub trait WithStartupShare {
    fn startup_share(&mut self, startup_share: &StartupShare) -> &mut Self;
}

impl WithStartupShare for Command {
    fn startup_share(&mut self, startup_share: &StartupShare) -> &mut Self {
        startup_share.apply_to(self);
        self
    }
}
