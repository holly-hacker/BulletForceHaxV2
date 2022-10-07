pub mod photon_data_type;
pub mod photon_message;
pub mod realtime;
pub(crate) mod utils;

pub use indexmap;
pub use ordered_float;

use thiserror::Error;

// TODO: perhaps add info on where the error occured?
macro_rules! check_remaining {
    ($bytes:ident, $min_bytes:expr) => {
        if $bytes.remaining() < $min_bytes {
            return Err(ReadError::NotEnoughBytesLeft);
        }
    };
}

pub(crate) use check_remaining;

/// An error that can occur when parsing a message
#[derive(Debug, Error)]
pub enum ReadError {
    #[error("not enough bytes left in the buffer")]
    NotEnoughBytesLeft,
    #[error("unexpected data was found: {0}")]
    UnexpectedData(&'static str),
    #[error("packet contained invalid magic number: {0:#02X}")]
    InvalidMagicNumber(u8),
    #[error("message type is unknown: {0:#02X}")]
    UnknownMessageType(u8),
    #[error("data type is unknown: {0:#02X}")]
    UnknownDataType(u8),
    #[error("the following functionality is not yet implemented: {0}")]
    Unimplemented(&'static str),
}

/// An error that can occur when writing a message
#[derive(Debug, Error)]
pub enum WriteError {
    // TODO: NotEnoughBytesLeft error, we currently panic if we write to a fixed-size buffer
    #[error("Items in array were not all of the same type")]
    UnhomogeneousArray,
    #[error("Key or value in typed dictionary did not match")]
    TypeMismatchInTypedDictionary,
    #[error("Value was too large: {0}")]
    ValueTooLarge(&'static str),
    #[error("the following functionality is not yet implemented: {0}")]
    Unimplemented(&'static str),
}
