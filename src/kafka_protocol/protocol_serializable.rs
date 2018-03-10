extern crate byteorder;

use self::byteorder::{BigEndian, ReadBytesExt};
use std::error::Error;
use std::io::Result as IOResult;
use std::io::Cursor;
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

pub fn de_array<T, F>(bytes: Vec<u8>, element_byte_size: usize, deserialize_t: F) -> ProtocolDeserializeResult<Vec<T>>
    where F: Fn(Vec<u8>) -> (T, Vec<u8>) {
    let array_size = de_i32(bytes[0..3].to_vec());

    array_size.and_then(|size| {
        let element_bytes = bytes[4..].to_vec();
        let elements = de_array_transform(element_bytes, deserialize_t);
        if elements.len() != (size as usize) {
            Err(DeserializeError::of(format!("Unexpected number of array elements. Expected {}, not {}.", size, elements.len())))
        } else {
            Ok(elements)
        }
    })
}

fn de_array_transform<T, F>(bytes: Vec<u8>, deserialize_t: F) -> Vec<T>
    where F: Fn(Vec<u8>) -> (T, Vec<u8>) {
    if bytes.len() <= 0 {
        vec![] as Vec<T>
    } else {
        let (t, leftover_bytes) = deserialize_t(bytes);
        let mut ts = vec![t];
        ts.append(&mut de_array_transform(leftover_bytes, deserialize_t));
        ts
    }
}

