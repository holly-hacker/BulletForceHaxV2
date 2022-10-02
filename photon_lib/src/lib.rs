pub mod photon_data_type;
pub mod photon_message;
pub(crate) mod utils;

use thiserror::Error;

// TODO: perhaps add info on where the error occured?
macro_rules! check_remaining {
    ($bytes:ident, $min_bytes:expr) => {
        if $bytes.remaining() < $min_bytes {
            return Err(ParseError::NotEnoughBytesLeft);
        }
    };
}

pub(crate) use check_remaining;

/// An error that can occur when parsing a message
#[derive(Debug, Error)]
pub enum ParseError {
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
