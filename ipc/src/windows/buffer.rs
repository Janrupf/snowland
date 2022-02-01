use crate::{IPCMessage, SnowlandIPCError};
use bincode::error::DecodeError;
use bincode::serde::Compat;

#[derive(Debug)]
pub struct IPCBuffer {
    inner: Vec<u8>, // TODO: A vec is probably not really efficient
    real_size: usize,
}

impl IPCBuffer {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            real_size: 0,
        }
    }

    pub fn flush(&mut self) {
        self.inner.clear();
        self.real_size = 0;
    }

    pub fn read_using<F>(&mut self, mut read_callback: F) -> Result<(), std::io::Error>
    where
        F: FnMut(&mut [u8]) -> Result<usize, std::io::Error>,
    {
        loop {
            let data_start = self.prepare_transfer_buffer();

            let read = match read_callback(data_start) {
                Ok(v) if v < 1 => return Ok(()),
                Ok(v) => v,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
                Err(err) => return Err(err),
            };

            self.mark_transferred(read);
        }
    }

    pub fn prepare_transfer_buffer(&mut self) -> &mut [u8] {
        let available = self.inner.len() - self.real_size;

        if available < 1024 {
            self.inner.resize(self.inner.len() + 1024, 0);
        }

        &mut self.inner[self.real_size..]
    }

    pub fn mark_transferred(&mut self, count: usize) {
        self.real_size += count;
    }

    pub fn decode_available_messages<C, R>(&mut self, config: C) -> Result<Vec<R>, DecodeError>
    where
        C: bincode::config::Config,
        R: IPCMessage + for<'de> serde::Deserialize<'de>,
    {
        let mut decoded = Vec::new();

        while self.real_size > 0 {
            let (message, read_size) =
                match bincode::decode_from_slice(&self.inner[0..self.real_size], config) {
                    Ok((Compat(v), s)) => (v, s),
                    Err(DecodeError::UnexpectedEnd) => {
                        log::trace!(
                            "Failed to decode more messages as not enough data is available"
                        );
                        break;
                    }
                    Err(err) => return Err(err),
                };

            decoded.push(message);

            self.real_size -= read_size;
            self.inner.drain(0..read_size);
        }

        Ok(decoded)
    }
}
