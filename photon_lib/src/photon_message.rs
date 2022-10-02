// Disable the clippy lint related to "manual" hash implementations for this file.
// Because `Indexmap` does not have a hash implementation, we need to specify our own hash function usind the
// `derivative` crate. Clippy does not like this because the (Partial)Eq and Hash function may become out of sync and
// provide conflicting results. I have verified the current implementations but it seems impossible or annoying to
// disable this lint for just derive attributes, so we disable it for the entire file.
#![allow(clippy::derive_hash_xor_eq)]

use bytes::{Buf, Bytes};
use derivative::Derivative;
use indexmap::IndexMap;

use crate::{check_remaining, photon_data_type::PhotonDataType, ParseError};

#[derive(Debug, PartialEq, Eq)]
pub enum PhotonMessage {
    // NOTE: 0 is most likely Init packet, but websocket does not seem to use it
    /// Message type 0x01, indicates that connection has been established.
    InitResponse,
    /// Message type 0x02
    OperationRequest(OperationRequest),
    /// Message type 0x03
    OperationResponse(OperationResponse),
    /// Message type 0x04
    EventData(EventData),
    /// Message type 0x05
    DisconnectMessage(DisconnectMessage),
    /// Message type 0x06
    InternalOperationRequest(OperationRequest),
    /// Message type 0x07
    InternalOperationResponse(OperationResponse),
    /// Message type 0x08
    Message(PhotonDataType),
    /// Message type 0x09, payload data does not seem to be used.
    RawMessage(Vec<u8>),
    /// S->C message with magic number 0xF0, the client will calculate roundtrip time and server time offset.
    PingResult(PingResult),
}

impl PhotonMessage {
    pub fn from_websocket_bytes(data: &mut Bytes) -> Result<PhotonMessage, ParseError> {
        if data.remaining() < 1 {
            return Err(ParseError::NotEnoughBytesLeft);
        }

        let magic_number = data.get_u8();

        match magic_number {
            0xF3 => {
                // photon checks if for `msg_type == 7 && op_code == 1 (ping)` and immediately handles the message if true
                // we dont need to do that, however

                Ok(PhotonMessage::from_bytes_f3(data)?)
            }
            0xF0 => Ok(PhotonMessage::PingResult(PingResult::from_bytes(data)?)),
            _ => Err(ParseError::InvalidMagicNumber(magic_number)),
        }
    }

    /// parse a message that uses magic number 0xF3
    fn from_bytes_f3(data: &mut Bytes) -> Result<Self, ParseError> {
        if data.remaining() < 2 {
            return Err(ParseError::NotEnoughBytesLeft);
        }

        let (msg_type, is_encrypted) = {
            let msg_byte = data.get_u8();
            (msg_byte & 0x7F, (msg_byte & 0x80) > 0)
        };

        if is_encrypted {
            return Err(ParseError::Unimplemented("encryption"));
        }

        match msg_type {
            1 => {
                _ = data.get_u8();
                Ok(PhotonMessage::InitResponse)
            }
            2 => Ok(PhotonMessage::OperationRequest(
                OperationRequest::from_bytes(data)?,
            )),
            3 => Ok(PhotonMessage::OperationResponse(
                OperationResponse::from_bytes(data)?,
            )),
            4 => Ok(PhotonMessage::EventData(EventData::from_bytes(data)?)),
            5 => Ok(PhotonMessage::DisconnectMessage(
                DisconnectMessage::from_bytes(data)?,
            )),
            6 => Ok(PhotonMessage::InternalOperationRequest(
                OperationRequest::from_bytes(data)?,
            )),
            7 => Ok(PhotonMessage::InternalOperationResponse(
                OperationResponse::from_bytes(data)?,
            )),
            8 => Ok(PhotonMessage::Message(PhotonDataType::from_bytes(data)?)),
            9 => Ok(PhotonMessage::RawMessage(
                data.copy_to_bytes(data.remaining()).to_vec(),
            )),
            _ => Err(ParseError::UnknownMessageType(msg_type)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PingResult {
    server_sent_time: i32,
    client_sent_time: i32,
}

impl PingResult {
    pub fn from_bytes(data: &mut Bytes) -> Result<Self, ParseError> {
        if data.remaining() < 8 {
            Err(ParseError::NotEnoughBytesLeft)
        } else {
            Ok(Self {
                server_sent_time: data.get_i32(),
                client_sent_time: data.get_i32(),
            })
        }
    }
}

#[derive(Debug, PartialEq, Eq, Derivative)]
#[derivative(Hash)]
pub struct OperationRequest {
    pub operation_code: u8,
    #[derivative(Hash(hash_with = "crate::utils::derive_utils::hash_indexmap"))]
    pub parameters: IndexMap<u8, PhotonDataType>,
}

impl OperationRequest {
    pub fn from_bytes(bytes: &mut Bytes) -> Result<Self, ParseError> {
        check_remaining!(bytes, 1);
        let operation_code = bytes.get_u8();

        let parameters = deserialize_parameter_dictionary(bytes)?;
        Ok(Self {
            operation_code,
            parameters,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Derivative)]
#[derivative(Hash)]
pub struct OperationResponse {
    pub operation_code: u8,
    pub return_code: i16,
    pub debug_message: Option<String>,
    #[derivative(Hash(hash_with = "crate::utils::derive_utils::hash_indexmap"))]
    pub parameters: IndexMap<u8, PhotonDataType>,
}

impl OperationResponse {
    pub fn from_bytes(bytes: &mut Bytes) -> Result<Self, ParseError> {
        check_remaining!(bytes, 3);
        let operation_code = bytes.get_u8();
        let return_code = bytes.get_i16();
        let debug_message = match PhotonDataType::from_bytes(bytes)? {
            PhotonDataType::String(s) => Some(s),
            PhotonDataType::Null => None,
            _ => {
                return Err(ParseError::UnexpectedData(
                    "expected string or null in operation response debug message",
                ))
            }
        };

        let parameters = deserialize_parameter_dictionary(bytes)?;
        Ok(Self {
            operation_code,
            return_code,
            debug_message,
            parameters,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Derivative)]
#[derivative(Hash)]
pub struct EventData {
    pub code: u8,
    #[derivative(Hash(hash_with = "crate::utils::derive_utils::hash_indexmap"))]
    pub parameters: IndexMap<u8, PhotonDataType>,
    // protocol 18 has a `sender` and `custom data` field, but we only support protocol 16 for now
}

impl EventData {
    pub fn from_bytes(bytes: &mut Bytes) -> Result<Self, ParseError> {
        check_remaining!(bytes, 1);
        let code = bytes.get_u8();

        let parameters = deserialize_parameter_dictionary(bytes)?;
        Ok(Self { code, parameters })
    }
}

#[derive(Debug, PartialEq, Eq, Derivative)]
#[derivative(Hash)]
pub struct DisconnectMessage {
    pub code: i16,
    pub debug_message: Option<String>,
    #[derivative(Hash(hash_with = "crate::utils::derive_utils::hash_indexmap"))]
    pub parameters: IndexMap<u8, PhotonDataType>,
}

impl DisconnectMessage {
    pub fn from_bytes(bytes: &mut Bytes) -> Result<Self, ParseError> {
        check_remaining!(bytes, 2);
        let code = bytes.get_i16();
        let debug_message = match PhotonDataType::from_bytes(bytes)? {
            PhotonDataType::String(s) => Some(s),
            PhotonDataType::Null => None,
            _ => {
                return Err(ParseError::UnexpectedData(
                    "expected string or null in operation response debug message",
                ))
            }
        };

        let parameters = deserialize_parameter_dictionary(bytes)?;
        Ok(Self {
            code,
            debug_message,
            parameters,
        })
    }
}

fn deserialize_parameter_dictionary(
    bytes: &mut Bytes,
) -> Result<IndexMap<u8, PhotonDataType>, ParseError> {
    check_remaining!(bytes, 2);
    let params_count = bytes.get_i16();
    let mut parameters = IndexMap::with_capacity(params_count as usize);
    for _ in 0..params_count {
        check_remaining!(bytes, 1);
        parameters.insert(bytes.get_u8(), PhotonDataType::from_bytes(bytes)?);
    }
    Ok(parameters)
}

#[cfg(test)]
mod tests {
    use indexmap::indexmap;

    use crate::{photon_data_type::PhotonDataType, photon_message::*};

    use super::PhotonMessage;

    macro_rules! test_message {
        ($name:ident, $hex:literal, $val:expr) => {
            paste::paste! {
                #[test]
            fn [<deserialize_ $name>]() {
                let mut bytes =
                    bytes::Bytes::from(hex::decode($hex).expect("valid hex data in test"));
                let val = $val;

                let deserialized = super::PhotonMessage::from_websocket_bytes(&mut bytes).unwrap();

                assert_eq!(deserialized, val);
            }
            }
        };
    }

    // TODO: add more tests cases

    test_message!(init_response, "f30100", PhotonMessage::InitResponse);

    // NOTE: noticed `f301 0073 000e 5265 7370 6f6e 7365 4f62 6a65 6374` in the wild

    test_message!(
        operation_request,
        "f302e50000",
        PhotonMessage::OperationRequest(OperationRequest {
            operation_code: 0xe5,
            parameters: indexmap!(),
        })
    );

    test_message!(
        operation_response,
        "f303e500002a0000",
        PhotonMessage::OperationResponse(OperationResponse {
            operation_code: 0xe5,
            return_code: 0,
            debug_message: None,
            parameters: indexmap!(),
        })
    );

    test_message!(
        event_data,
        "f304e20003e36900000011e5690000006ee46900000016",
        PhotonMessage::EventData(EventData {
            code: 0xe2,
            parameters: indexmap! {
                0xe3 => PhotonDataType::Integer(0x11),
                0xe5 => PhotonDataType::Integer(0x6e),
                0xe4 => PhotonDataType::Integer(0x16),
            }
        })
    );

    test_message!(
        internal_operation_request,
        "f3060100010169000330de",
        PhotonMessage::InternalOperationRequest(OperationRequest {
            operation_code: 1,
            parameters: indexmap! {
                1 => PhotonDataType::Integer(0x330de),
            }
        })
    );

    test_message!(
        internal_operation_response,
        "f3070100002a0002016900002efd026938c2510f",
        PhotonMessage::InternalOperationResponse(OperationResponse {
            operation_code: 1,
            return_code: 0,
            debug_message: None,
            parameters: indexmap! {
                1 => PhotonDataType::Integer(0x2efd),
                2 => PhotonDataType::Integer(0x38c2510f),
            }
        })
    );
}
