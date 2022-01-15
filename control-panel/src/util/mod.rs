//! General utility module

mod dart_structure;
pub use dart_structure::*;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Helper function to serialize an arbitrary value into json and then back into another value.
pub fn reserialize<'a, I: Serialize, O: Deserialize<'a>>(input: I) -> Result<O, ReserializeError> {
    let intermediate =
        serde_json::to_value(input).map_err(ReserializeError::SerializationFailed)?;
    log::trace!("Reserialization intermediate: {:#?}", intermediate);
    let output = O::deserialize(intermediate).map_err(ReserializeError::DeserializationFailed)?;

    Ok(output)
}

/// Errors that may occur while transforming a value using [`reserialize`].
#[derive(Debug, Error)]
pub enum ReserializeError {
    #[error("failed to serialize value: {0}")]
    SerializationFailed(serde_json::Error),

    #[error("failed to deserialize value: {0}")]
    DeserializationFailed(serde_json::Error),
}
