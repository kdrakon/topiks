extern crate byteorder;

use self::byteorder::{BigEndian, ReadBytesExt};
use std::error::Error;
use std::io::Result as IOResult;
use std::io::Cursor;
use std::str::from_utf8;
use super::protocol_response::*;

/// If implemented, a struct/enum can be sent on the wire to a
/// Kafka broker.
///
pub trait ProtocolSerializable {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult;
}

pub type ProtocolSerializeResult = IOResult<Vec<u8>>;

/// If implemented, a Vec<u8> can be read from a Kafka broker
/// into a type T
///
pub trait ProtocolDeserializable<T> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<T>;
}

pub type ProtocolDeserializeResult<T> = Result<T, DeserializeError>;
pub struct DeserializeError { error: String }
impl DeserializeError {
    pub fn of(error: String) -> DeserializeError { DeserializeError { error } }
}

// Deserializer Functions
pub fn de_i32(bytes: Vec<u8>) -> ProtocolDeserializeResult<i32> {
    Cursor::new(bytes).read_i32::<BigEndian>().map_err(|e| DeserializeError::of(e.description().to_string()))
}

pub fn de_i16(bytes: Vec<u8>) -> ProtocolDeserializeResult<i16> {
    Cursor::new(bytes).read_i16::<BigEndian>().map_err(|e| DeserializeError::of(e.description().to_string()))
}

pub type DynamicType<T> = (T, Vec<u8>); // Vec<u8> == remaining bytes after

pub fn de_array<T, F>(bytes: Vec<u8>, deserialize_t: F) -> ProtocolDeserializeResult<DynamicType<Vec<T>>>
    where F: Fn(Vec<u8>) -> (T, Vec<u8>) {

    let array_size = de_i32(bytes[0..3].to_vec());

    array_size.and_then(|expected_elements| {
        let element_bytes = bytes[4..].to_vec();
        let (elements, remaining_bytes) = de_array_transform(element_bytes, expected_elements, deserialize_t);
        if elements.len() != (expected_elements as usize) {
            Err(DeserializeError::of(format!("Unexpected number of array elements. Expected {}, not {}.", expected_elements, elements.len())))
        } else {
            Ok((elements, remaining_bytes))
        }
    })
}

fn de_array_transform<T, F>(bytes: Vec<u8>, elements: i32, deserialize_t: F) -> DynamicType<Vec<T>>
    where F: Fn(Vec<u8>) -> (T, Vec<u8>) {

    if elements <= 0 {
        (vec![] as Vec<T>, bytes)
    } else {
        let (t, leftover_bytes) = deserialize_t(bytes);
        let (mut next_ts, leftover_bytes) = de_array_transform(leftover_bytes, elements - 1, deserialize_t);
        let mut ts = vec![t];
        ts.append(&mut next_ts);
        (ts, leftover_bytes)
    }
}

fn de_string(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicType<String>> {
    de_i16(bytes[0..1].to_vec()).and_then(|string_size|{
        let usize_string_size = string_size as usize;
        let remaining_bytes = bytes[usize_string_size + 1..].to_vec();
        let string_bytes = &bytes[2..usize_string_size];

        if let Ok(string) = from_utf8(string_bytes) {
            Ok((String::from(string), remaining_bytes))
        } else {
            Err(DeserializeError::of(String::from("Failed to deserialize string")))
        }
    })
}

fn de_opt_string(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicType<Option<String>>> {
    unimplemented!()
}

