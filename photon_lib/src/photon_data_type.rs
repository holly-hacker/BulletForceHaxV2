use bytes::{Buf, Bytes};
use indexmap::IndexMap;
use ordered_float::OrderedFloat;
use std::hash::Hash;

use crate::{
    check_remaining,
    photon_message::{EventData, OperationRequest, OperationResponse},
    ParseError,
};

/// A serialized .NET object
#[derive(Debug, Default, PartialEq, Eq)]
pub enum PhotonDataType {
    #[default]
    /// Data type 0x2A, represents .NET's `null`
    Null,
    /// Data type 0x44, holds an `IDictionary<object, object>`
    Dictionary(IndexMap<PhotonDataType, PhotonDataType>),
    /// Data type 0x61, holds a `string[]`.
    StringArray(Vec<String>),
    /// Data type 0x62, holds a `byte`
    Byte(u8),
    /// Data type 0x63, holds an `object`. This uses a deserialization function that is provided by the game.
    Custom(u8, Vec<u8>),
    /// Data type 0x64, holds a `double`
    Double(OrderedFloat<f64>),
    /// Data type 0x65, holds [EventData]
    EventData(EventData),
    /// Data type 0x66, holds a `float`
    Float(OrderedFloat<f32>),
    /// Data type 0x68, holds a photon Hashtable. This hashtable aims to mimic `System.Collections.Hashtable`.
    Hashtable(IndexMap<PhotonDataType, PhotonDataType>),
    /// Data type 0x69, holds an `int`
    Integer(i32),
    /// Data type 0x6B, holds a `short`
    Short(i16),
    /// Data type 0x6C, holds a `long`
    Long(i64),
    /// Data type 0x6E, holds an `int[]`
    IntArray(Vec<i32>),
    /// Data type 0x6F, holds a `bool`
    Boolean(bool),
    /// Data type 0x70, holds an [OperationResponse]
    OperationResponse(OperationResponse),
    /// Data type 0x71, holds an [OperationRequest]
    OperationRequest(OperationRequest),
    /// Data type 0x73, holds a `string`
    String(String),
    /// Data type 0x78, holds a `byte[]`
    ByteArray(Vec<u8>),
    /// Data type 0x79, holds an `Array`
    Array(Vec<PhotonDataType>),
    /// Data type 0x7A, holds an `object[]`
    ObjectArray(Vec<PhotonDataType>),
}

impl PhotonDataType {
    pub fn from_bytes(bytes: &mut Bytes) -> Result<PhotonDataType, ParseError> {
        check_remaining!(bytes, 1);

        let data_type = bytes.get_u8();
        Self::from_bytes_with_type(bytes, data_type)
    }

    pub fn from_bytes_with_type(
        bytes: &mut Bytes,
        data_type: u8,
    ) -> Result<PhotonDataType, ParseError> {
        match data_type {
            0 | 0x2A => Ok(PhotonDataType::Null),
            0x44 => {
                check_remaining!(bytes, 4);
                // NOTE: implementation does not allow 0x44 or 0x69 as key or value
                let key_type = bytes.get_u8();
                let val_type = bytes.get_u8();
                let len = bytes.get_i16();

                let read_key = key_type == 0 || key_type == 0x2A;
                let read_val = val_type == 0 || val_type == 0x2A;

                let mut map = indexmap::IndexMap::new();
                for _ in 0..len {
                    let key = match read_key {
                        true => Self::from_bytes(bytes)?,
                        false => Self::from_bytes_with_type(bytes, key_type)?,
                    };
                    let val = match read_val {
                        true => Self::from_bytes(bytes)?,
                        false => Self::from_bytes_with_type(bytes, val_type)?,
                    };

                    if key != PhotonDataType::Null {
                        map.insert(key, val);
                    }
                }

                Ok(PhotonDataType::Dictionary(map))
            }
            0x61 => {
                check_remaining!(bytes, 2);
                let len = bytes.get_i16();
                let v = if len > 0 {
                    let mut v = Vec::with_capacity(len as usize);
                    for _ in 0..len {
                        match Self::from_bytes_with_type(bytes, 0x73)? {
                            PhotonDataType::String(s) => v.push(s),
                            _ => unreachable!(),
                        }
                    }
                    v
                } else {
                    vec![]
                };
                Ok(PhotonDataType::StringArray(v))
            }
            0x62 => {
                check_remaining!(bytes, 1);
                Ok(PhotonDataType::Byte(bytes.get_u8()))
            }
            0x63 => {
                check_remaining!(bytes, 3);
                let type_code = bytes.get_u8();
                let len = bytes.get_i16();
                if len < 0 {
                    return Err(ParseError::UnexpectedDataError(
                        "negative length for custom data",
                    ));
                } else {
                    check_remaining!(bytes, len as usize);
                    let mut v = vec![0u8; len as usize];
                    bytes.copy_to_slice(&mut v);
                    Ok(PhotonDataType::Custom(type_code, v))
                }
            }
            0x64 => {
                check_remaining!(bytes, 8);
                Ok(PhotonDataType::Double(bytes.get_f64().into()))
            }
            0x65 => Ok(PhotonDataType::EventData(EventData::from_bytes(bytes)?)),
            0x66 => {
                check_remaining!(bytes, 4);
                Ok(PhotonDataType::Float(bytes.get_f32().into()))
            }
            0x68 => {
                check_remaining!(bytes, 4);
                // NOTE: implementation does not allow 0x44 or 0x69 as key or value
                let len = bytes.get_i16();

                let mut map = indexmap::IndexMap::new();
                for _ in 0..len {
                    let key = Self::from_bytes(bytes)?;
                    let val = Self::from_bytes(bytes)?;

                    if key != PhotonDataType::Null {
                        map.insert(key, val);
                    }
                }

                Ok(PhotonDataType::Hashtable(map))
            }
            0x69 => {
                check_remaining!(bytes, 4);
                Ok(PhotonDataType::Integer(bytes.get_i32()))
            }
            0x6B => {
                check_remaining!(bytes, 2);
                Ok(PhotonDataType::Short(bytes.get_i16()))
            }
            0x6C => {
                check_remaining!(bytes, 8);
                Ok(PhotonDataType::Long(bytes.get_i64()))
            }
            0x6E => {
                check_remaining!(bytes, 4);
                let len = bytes.get_i32();
                let v = if len > 0 {
                    let mut v = Vec::with_capacity(len as usize);
                    for _ in 0..len {
                        check_remaining!(bytes, 4);
                        v.push(bytes.get_i32());
                    }
                    v
                } else {
                    vec![]
                };
                Ok(PhotonDataType::IntArray(v))
            }
            0x6F => {
                check_remaining!(bytes, 1);
                Ok(PhotonDataType::Boolean(bytes.get_u8() != 0))
            }
            0x70 => Ok(PhotonDataType::OperationResponse(
                OperationResponse::from_bytes(bytes)?,
            )),
            0x71 => Ok(PhotonDataType::OperationRequest(
                OperationRequest::from_bytes(bytes)?,
            )),
            0x73 => {
                check_remaining!(bytes, 2);
                let len = bytes.get_i16();
                let str = if len > 0 {
                    check_remaining!(bytes, len as usize);
                    let mut buffer = vec![0u8; len as usize];
                    bytes.copy_to_slice(&mut buffer);

                    // NOTE: System.Text.Encoding.UTF8.GetString will replace invalid unicode with �, so we imitate
                    // that behavior.
                    let str = String::from_utf8_lossy(&buffer);
                    str.to_string()
                } else if len == 0 {
                    String::new()
                } else {
                    return Err(ParseError::UnexpectedDataError("string length less than 0"));
                };

                Ok(PhotonDataType::String(str))
            }
            0x78 => {
                check_remaining!(bytes, 4);
                let len = bytes.get_i32();
                if len < 0 {
                    return Err(ParseError::UnexpectedDataError("byte[] lenght less than 0"));
                }

                check_remaining!(bytes, len as usize);
                let mut v = vec![0u8; len as usize];
                bytes.copy_to_slice(&mut v);

                Ok(PhotonDataType::ByteArray(v))
            }
            0x79 => {
                check_remaining!(bytes, 3);
                let len = bytes.get_i16();
                let data_type = bytes.get_u8();

                let v = if len > 0 {
                    let mut vec = Vec::with_capacity(len as usize);

                    for _ in 0..len {
                        vec.push(Self::from_bytes_with_type(bytes, data_type)?);
                    }

                    vec
                } else {
                    vec![]
                };

                Ok(PhotonDataType::Array(v))
            }
            0x7A => {
                check_remaining!(bytes, 2);
                let len = bytes.get_i16();

                if len < 0 {
                    return Err(ParseError::UnexpectedDataError(
                        "object[] length less than 0",
                    ));
                }

                let mut v = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    v.push(Self::from_bytes(bytes)?);
                }

                Ok(PhotonDataType::ObjectArray(v))
            }
            _ => Err(ParseError::UnknownDataType(data_type)),
        }
    }
}

// NOTE: could also have done this with the `derivative` crate:
// https://mcarton.github.io/rust-derivative/latest/Hash.html
impl Hash for PhotonDataType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            PhotonDataType::Null => (),
            PhotonDataType::Dictionary(x) => {
                for (key, value) in x {
                    key.hash(state);
                    value.hash(state);
                }
            }
            PhotonDataType::StringArray(x) => x.hash(state),
            PhotonDataType::Byte(x) => x.hash(state),
            PhotonDataType::Custom(x, y) => {
                x.hash(state);
                y.hash(state);
            }
            PhotonDataType::Double(x) => x.hash(state),
            PhotonDataType::EventData(x) => x.hash(state),
            PhotonDataType::Float(x) => x.hash(state),
            PhotonDataType::Hashtable(x) => {
                for (key, value) in x {
                    key.hash(state);
                    value.hash(state);
                }
            }
            PhotonDataType::Integer(x) => x.hash(state),
            PhotonDataType::Short(x) => x.hash(state),
            PhotonDataType::Long(x) => x.hash(state),
            PhotonDataType::IntArray(x) => x.hash(state),
            PhotonDataType::Boolean(x) => x.hash(state),
            PhotonDataType::OperationResponse(x) => x.hash(state),
            PhotonDataType::OperationRequest(x) => x.hash(state),
            PhotonDataType::String(x) => x.hash(state),
            PhotonDataType::ByteArray(x) => x.hash(state),
            PhotonDataType::Array(x) => x.hash(state),
            PhotonDataType::ObjectArray(x) => x.hash(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::photon_message::*;

    use super::PhotonDataType;

    macro_rules! generate_read_write_test {
        ($name: ident, $val: expr, $hex: expr) => {
            paste::paste! {
                #[test]
                fn [<deserialize_ $name>]() {
                    let mut bytes =
                        bytes::Bytes::from(hex::decode($hex).expect("valid hex data in test"));
                    let val = $val;

                    let deserialized = super::PhotonDataType::from_bytes(&mut bytes).unwrap();

                    assert_eq!(deserialized, val);
                }
            }
        };
    }

    #[test]
    fn deserialize_00_to_null() {
        assert_eq!(
            PhotonDataType::from_bytes(&mut bytes::Bytes::from(&[0x00u8][..])).unwrap(),
            PhotonDataType::Null
        );
    }

    generate_read_write_test!(null, PhotonDataType::Null, "2a");
    generate_read_write_test!(bool_true, PhotonDataType::Boolean(true), "6f01");
    generate_read_write_test!(bool_false, PhotonDataType::Boolean(false), "6f00");
    generate_read_write_test!(u8, PhotonDataType::Byte(0x90), "6290");
    generate_read_write_test!(s16, PhotonDataType::Short(-1337), "6BFAC7");
    generate_read_write_test!(s32, PhotonDataType::Integer(-559038737), "69DEADBEEF");
    generate_read_write_test!(
        s64,
        PhotonDataType::Long(-3886136854700967234),
        "6cCA11AB1ECAFEBABE"
    );
    generate_read_write_test!(f32, PhotonDataType::Float(42f32.into()), "6642280000");
    generate_read_write_test!(
        f64,
        PhotonDataType::Double(13.37f64.into()),
        "64402abd70a3d70a3d"
    );
    generate_read_write_test!(string, PhotonDataType::String("abc".into()), "730003616263");
    generate_read_write_test!(
        string_unicode,
        PhotonDataType::String("abc»d".into()),
        "730006616263c2bb64"
    );
    generate_read_write_test!(
        byte_array,
        PhotonDataType::ByteArray(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        "7800000004DEADBEEF"
    );
    generate_read_write_test!(
        int_array,
        PhotonDataType::IntArray(vec![-559038737, -889275714]),
        "6E00000002DEADBEEFCAFEBABE"
    );
    generate_read_write_test!(
        string_array,
        PhotonDataType::StringArray(vec!["abc".into(), "".into()]),
        "61000200036162630000"
    );
    generate_read_write_test!(
        array,
        PhotonDataType::Array(vec![
            PhotonDataType::Boolean(true),
            PhotonDataType::Boolean(false),
            PhotonDataType::Boolean(true)
        ]),
        "7900036F010001"
    );
    generate_read_write_test!(
        object_array,
        PhotonDataType::ObjectArray(vec![
            PhotonDataType::String("abc".into()),
            PhotonDataType::Null,
            PhotonDataType::Short(0x123)
        ]),
        "7A00037300036162632A6B0123"
    );

    // hashtable can only have 1 item because order is not deterministic
    generate_read_write_test!(
        hashtable,
        PhotonDataType::Hashtable(
            indexmap::indexmap! { PhotonDataType::Byte(0xFF) => PhotonDataType::Null, }
        ),
        "68000162FF2A"
    );

    generate_read_write_test!(
        dictionary_byte_string,
        PhotonDataType::Dictionary(indexmap::indexmap! {
            PhotonDataType::Byte(0x01) => PhotonDataType::String("one".into()),
            PhotonDataType::Byte(0x02) => PhotonDataType::String("two".into()),
        }),
        "44627300020100036f6e6502000374776f"
    );

    generate_read_write_test!(
        dictionary_untyped,
        PhotonDataType::Dictionary(indexmap::indexmap! {
            PhotonDataType::Byte(0x00) => PhotonDataType::Short(0x1234),
            PhotonDataType::String("a".into()) => PhotonDataType::Byte(0xFF),
        }),
        "44002A000262006B12347300016162FF"
    );

    generate_read_write_test!(
        event_data,
        PhotonDataType::EventData(EventData {
            code: 0x12,
            parameters: indexmap::indexmap! {
                0x01 => PhotonDataType::Short(0x1234),
                0xFF => PhotonDataType::Byte(0xFF),
            }
        }),
        "65120002016B1234FF62FF"
    );

    generate_read_write_test!(
        operation_response,
        PhotonDataType::OperationResponse(OperationResponse {
            operation_code: 0x12,
            return_code: -1,
            debug_message: Some("test".into()),
            parameters: indexmap::indexmap! {
                0x01 => PhotonDataType::Short(0x1234),
                0xFF => PhotonDataType::Byte(0xFF),
            }
        }),
        "7012FFFF730004746573740002016B1234FF62FF"
    );

    generate_read_write_test!(
        operation_request,
        PhotonDataType::OperationRequest(OperationRequest {
            operation_code: 0x12,
            parameters: indexmap::indexmap! {
                0x01 => PhotonDataType::Short(0x1234),
                0xFF => PhotonDataType::Byte(0xFF),
            }
        }),
        "71120002016B1234FF62FF"
    );

    // NOTE: original code had tests to detect vec2, vec3, quaternion, etc. we're not supporting that this time
    generate_read_write_test!(
        other_custom,
        PhotonDataType::Custom(15, vec![0xDE, 0xAD, 0xBE, 0xEF]),
        "630F0004DEADBEEF"
    );
}
