// Disable the clippy lint related to "manual" hash implementations for this file.
// Because `Indexmap` does not have a hash implementation, we need to specify our own hash function usind the
// `derivative` crate. Clippy does not like this because the (Partial)Eq and Hash function may become out of sync and
// provide conflicting results. I have verified the current implementations but it seems impossible or annoying to
// disable this lint for just derive attributes, so we disable it for the entire file.
#![allow(clippy::derive_hash_xor_eq)]

use std::cmp::Ordering;

use bytes::{Buf, BufMut};
use derivative::Derivative;
use indexmap::IndexMap;
use ordered_float::OrderedFloat;

// use std::hash::Hash;

use crate::{
    check_remaining,
    photon_message::{EventData, OperationRequest, OperationResponse},
    ReadError, WriteError,
};

/// A serialized .NET object
#[derive(Debug, Default, PartialEq, Eq, Derivative)]
#[derivative(Hash)]
pub enum PhotonDataType {
    #[default]
    /// Data type 0x2A, represents .NET's `null`
    Null,
    /// Data type 0x44, holds a `Dictionary<TKey, TValue>`. Because this dictionary is generic, we need to store the key and value kind as well.
    Dictionary(
        (u8, u8),
        #[derivative(Hash(hash_with = "crate::utils::derive_utils::hash_indexmap"))]
        IndexMap<PhotonDataType, PhotonDataType>,
    ),
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
    Hashtable(
        #[derivative(Hash(hash_with = "crate::utils::derive_utils::hash_indexmap"))]
        IndexMap<PhotonDataType, PhotonDataType>,
    ),
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
    /// Data type 0x79, holds an `Array`. Elements must be of the same type.
    Array(Vec<PhotonDataType>),
    /// Data type 0x7A, holds an `object[]`
    ObjectArray(Vec<PhotonDataType>),
}

impl PhotonDataType {
    pub fn from_bytes(bytes: &mut impl Buf) -> Result<PhotonDataType, ReadError> {
        check_remaining!(bytes, 1);

        let data_type = bytes.get_u8();
        Self::from_bytes_with_type(bytes, data_type)
    }

    pub fn from_bytes_with_type(
        bytes: &mut impl Buf,
        data_type: u8,
    ) -> Result<PhotonDataType, ReadError> {
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

                Ok(PhotonDataType::Dictionary((key_type, val_type), map))
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
                    Err(ReadError::UnexpectedData("negative length for custom data"))
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
                check_remaining!(bytes, 2);
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
                let str = match len.cmp(&0) {
                    Ordering::Greater => {
                        check_remaining!(bytes, len as usize);
                        let mut buffer = vec![0u8; len as usize];
                        bytes.copy_to_slice(&mut buffer);

                        // NOTE: System.Text.Encoding.UTF8.GetString will replace invalid unicode with �, so we imitate
                        // that behavior.
                        let str = String::from_utf8_lossy(&buffer);
                        str.to_string()
                    }
                    Ordering::Equal => String::new(),
                    // this seems inconsistent with other branches but this is what the original code would do
                    Ordering::Less => {
                        return Err(ReadError::UnexpectedData("string length less than 0"));
                    }
                };

                Ok(PhotonDataType::String(str))
            }
            0x78 => {
                check_remaining!(bytes, 4);
                let len = bytes.get_i32();
                if len < 0 {
                    return Err(ReadError::UnexpectedData("byte[] length less than 0"));
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
                    return Err(ReadError::UnexpectedData("object[] length less than 0"));
                }

                let mut v = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    v.push(Self::from_bytes(bytes)?);
                }

                Ok(PhotonDataType::ObjectArray(v))
            }
            _ => Err(ReadError::UnknownDataType(data_type)),
        }
    }

    pub fn to_bytes(&self, buf: &mut impl BufMut) -> Result<(), WriteError> {
        buf.put_u8(self.get_type_byte());

        self.to_bytes_without_type_byte(buf)
    }

    pub fn to_bytes_without_type_byte(&self, buf: &mut impl BufMut) -> Result<(), WriteError> {
        match self {
            PhotonDataType::Null => (),
            PhotonDataType::Dictionary((key_type, val_type), d) => {
                buf.put_u8(*key_type);
                buf.put_u8(*val_type);

                if d.len() > i16::MAX as usize {
                    return Err(WriteError::ValueTooLarge("Custom Data"));
                }

                buf.put_i16(d.len() as i16);

                let write_key = *key_type == 0 || *key_type == 0x2A;
                let write_val = *val_type == 0 || *val_type == 0x2A;

                for (k, v) in d {
                    match write_key {
                        true => k.to_bytes(buf)?,
                        false if k.get_type_byte() != *key_type => {
                            return Err(WriteError::TypeMismatchInTypedDictionary)
                        }
                        false => k.to_bytes_without_type_byte(buf)?,
                    };

                    match write_val {
                        true => v.to_bytes(buf)?,
                        false if v.get_type_byte() != *val_type => {
                            return Err(WriteError::TypeMismatchInTypedDictionary)
                        }
                        false => v.to_bytes_without_type_byte(buf)?,
                    };
                }
            }
            PhotonDataType::StringArray(a) => {
                if a.len() > i16::MAX as usize {
                    return Err(WriteError::ValueTooLarge("Custom Data"));
                }
                buf.put_i16(a.len() as i16);

                for s in a {
                    let len = s.as_bytes().len();
                    if len > i16::MAX as usize {
                        return Err(WriteError::ValueTooLarge("String"));
                    }
                    buf.put_i16(len as i16);
                    buf.put_slice(s.as_bytes());
                }
            }
            PhotonDataType::Byte(b) => buf.put_u8(*b),
            PhotonDataType::Custom(type_code, v) => {
                buf.put_u8(*type_code);

                if v.len() > i16::MAX as usize {
                    return Err(WriteError::ValueTooLarge("Custom Data"));
                }

                buf.put_i16(v.len() as i16);
                buf.put_slice(v);
            }
            PhotonDataType::Double(d) => buf.put_f64(d.0),
            PhotonDataType::EventData(d) => d.to_bytes(buf)?,
            PhotonDataType::Float(f) => buf.put_f32(f.0),
            PhotonDataType::Hashtable(t) => {
                if t.len() > i16::MAX as usize {
                    return Err(WriteError::ValueTooLarge("String"));
                }

                buf.put_i16(t.len() as i16);

                for (k, v) in t {
                    k.to_bytes(buf)?;
                    v.to_bytes(buf)?;
                }
            }
            PhotonDataType::Integer(i) => buf.put_i32(*i),
            PhotonDataType::Short(s) => buf.put_i16(*s),
            PhotonDataType::Long(l) => buf.put_i64(*l),
            PhotonDataType::IntArray(v) => {
                if v.len() > i32::MAX as usize {
                    return Err(WriteError::ValueTooLarge("String"));
                }
                buf.put_i32(v.len() as i32);

                for &i in v {
                    buf.put_i32(i);
                }
            }
            PhotonDataType::Boolean(b) => buf.put_u8(if *b { 1 } else { 0 }),
            PhotonDataType::OperationResponse(r) => r.to_bytes(buf)?,
            PhotonDataType::OperationRequest(r) => r.to_bytes(buf)?,
            PhotonDataType::String(s) => {
                let len = s.as_bytes().len();
                if len > i16::MAX as usize {
                    return Err(WriteError::ValueTooLarge("String"));
                }
                buf.put_i16(len as i16);
                buf.put_slice(s.as_bytes());
            }
            PhotonDataType::ByteArray(v) => {
                if v.len() > i32::MAX as usize {
                    return Err(WriteError::ValueTooLarge("ByteArray"));
                }
                buf.put_i32(v.len() as i32);

                buf.put_slice(v);
            }
            PhotonDataType::Array(v) => {
                if v.len() > i16::MAX as usize {
                    return Err(WriteError::ValueTooLarge("Array"));
                }
                buf.put_i16(v.len() as i16);

                let type_byte = match v.get(0) {
                    Some(i) => i.get_type_byte(),
                    None => PhotonDataType::Null.get_type_byte(),
                };
                buf.put_u8(type_byte);

                for item in v {
                    if item.get_type_byte() != type_byte {
                        return Err(WriteError::UnhomogeneousArray);
                    }
                    item.to_bytes_without_type_byte(buf)?;
                }
            }
            PhotonDataType::ObjectArray(v) => {
                if v.len() > i16::MAX as usize {
                    return Err(WriteError::ValueTooLarge("ObjectArray"));
                }
                buf.put_i16(v.len() as i16);

                for item in v {
                    item.to_bytes(buf)?;
                }
            }
        }

        Ok(())
    }

    pub fn get_type_byte(&self) -> u8 {
        match self {
            PhotonDataType::Null => 0x2A,
            PhotonDataType::Dictionary(_, _) => 0x44,
            PhotonDataType::StringArray(_) => 0x61,
            PhotonDataType::Byte(_) => 0x62,
            PhotonDataType::Custom(_, _) => 0x63,
            PhotonDataType::Double(_) => 0x64,
            PhotonDataType::EventData(_) => 0x65,
            PhotonDataType::Float(_) => 0x66,
            PhotonDataType::Hashtable(_) => 0x68,
            PhotonDataType::Integer(_) => 0x69,
            PhotonDataType::Short(_) => 0x6B,
            PhotonDataType::Long(_) => 0x6C,
            PhotonDataType::IntArray(_) => 0x6E,
            PhotonDataType::Boolean(_) => 0x6F,
            PhotonDataType::OperationResponse(_) => 0x70,
            PhotonDataType::OperationRequest(_) => 0x71,
            PhotonDataType::String(_) => 0x73,
            PhotonDataType::ByteArray(_) => 0x78,
            PhotonDataType::Array(_) => 0x79,
            PhotonDataType::ObjectArray(_) => 0x7A,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::photon_message::*;

    use super::PhotonDataType;

    macro_rules! generate_test {
        ($name: ident, $val: expr, $hex: expr) => {
            paste::paste! {
                #[test]
                fn [<deserialize_ $name>]() {
                    let mut bytes: &[u8] = &hex::decode($hex).expect("valid hex data in test");
                    let val = $val;

                    let deserialized = super::PhotonDataType::from_bytes(&mut bytes).unwrap();

                    assert_eq!(deserialized, val);
                }

                #[test]
                fn [<serialize_ $name>]() {
                    use super::PhotonDataType;

                    let bytes = hex::decode($hex).expect("valid hex data in test");
                    let val = $val;

                    let mut buf = vec![];
                    val.to_bytes(&mut buf).unwrap();

                    assert_eq!(buf, bytes);
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

    generate_test!(null, PhotonDataType::Null, "2a");
    generate_test!(bool_true, PhotonDataType::Boolean(true), "6f01");
    generate_test!(bool_false, PhotonDataType::Boolean(false), "6f00");
    generate_test!(u8, PhotonDataType::Byte(0x90), "6290");
    generate_test!(s16, PhotonDataType::Short(-1337), "6BFAC7");
    generate_test!(s32, PhotonDataType::Integer(-559038737), "69DEADBEEF");
    generate_test!(
        s64,
        PhotonDataType::Long(-3886136854700967234),
        "6cCA11AB1ECAFEBABE"
    );
    generate_test!(f32, PhotonDataType::Float(42f32.into()), "6642280000");
    generate_test!(
        f64,
        PhotonDataType::Double(13.37f64.into()),
        "64402abd70a3d70a3d"
    );
    generate_test!(string, PhotonDataType::String("abc".into()), "730003616263");
    generate_test!(
        string_unicode,
        PhotonDataType::String("abc»d".into()),
        "730006616263c2bb64"
    );
    generate_test!(
        byte_array,
        PhotonDataType::ByteArray(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        "7800000004DEADBEEF"
    );
    generate_test!(
        int_array,
        PhotonDataType::IntArray(vec![-559038737, -889275714]),
        "6E00000002DEADBEEFCAFEBABE"
    );
    generate_test!(
        string_array,
        PhotonDataType::StringArray(vec!["abc".into(), "".into()]),
        "61000200036162630000"
    );
    generate_test!(
        array,
        PhotonDataType::Array(vec![
            PhotonDataType::Boolean(true),
            PhotonDataType::Boolean(false),
            PhotonDataType::Boolean(true)
        ]),
        "7900036F010001"
    );
    generate_test!(
        object_array,
        PhotonDataType::ObjectArray(vec![
            PhotonDataType::String("abc".into()),
            PhotonDataType::Null,
            PhotonDataType::Short(0x123)
        ]),
        "7A00037300036162632A6B0123"
    );

    // hashtable can only have 1 item because order is not deterministic
    generate_test!(
        hashtable,
        PhotonDataType::Hashtable(
            indexmap::indexmap! { PhotonDataType::Byte(0xFF) => PhotonDataType::Null, }
        ),
        "68000162FF2A"
    );

    generate_test!(
        dictionary_byte_string,
        PhotonDataType::Dictionary(
            (0x62, 0x73),
            indexmap::indexmap! {
                PhotonDataType::Byte(0x01) => PhotonDataType::String("one".into()),
                PhotonDataType::Byte(0x02) => PhotonDataType::String("two".into()),
            }
        ),
        "44627300020100036f6e6502000374776f"
    );

    generate_test!(
        dictionary_untyped,
        PhotonDataType::Dictionary(
            (0, 0),
            indexmap::indexmap! {
                PhotonDataType::Byte(0x00) => PhotonDataType::Short(0x1234),
                PhotonDataType::String("a".into()) => PhotonDataType::Byte(0xFF),
            }
        ),
        "440000000262006B12347300016162FF"
    );

    generate_test!(
        dictionary_typed_key,
        PhotonDataType::Dictionary(
            (0x62, 0),
            indexmap::indexmap! {
                PhotonDataType::Byte(0x00) => PhotonDataType::Short(0x1234),
                PhotonDataType::Byte(0x01) => PhotonDataType::Byte(0xFF),
            }
        ),
        "4462000002006B12340162FF"
    );

    generate_test!(
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

    generate_test!(
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

    generate_test!(
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
    generate_test!(
        other_custom,
        PhotonDataType::Custom(15, vec![0xDE, 0xAD, 0xBE, 0xEF]),
        "630F0004DEADBEEF"
    );
}
