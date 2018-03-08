extern crate byteorder;

use super::protocol_response::*;
use self::byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use std::error::Error;

/// If implemented, a struct/enum can be sent on the wire to a
/// Kafka broker.
///
pub trait ProtocolSerializable {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult;
}

pub type ProtocolSerializeResult = Result<Vec<u8>, ProtocolSerializeError>; // TODO replace Result with Result from std::io, which takes one generic type

#[derive(Debug)]
pub struct ProtocolSerializeError(pub String);

/// If implemented, a Vec<u8> can be read from a Kafka broker
/// into a type T
///
pub trait ProtocolDeserializable<T> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<T>;
}

pub type ProtocolDeserializeResult<T> = Result<T, ProtocolDeserializeError>; // TODO replace Result with Result from std::io, which takes one generic type

#[derive(Debug)]
pub struct ProtocolDeserializeError(pub String);

// Deserializer Functions
pub fn de_i32(bytes: Vec<u8>) -> ProtocolDeserializeResult<i32> {
    Cursor::new(bytes).read_i32::<BigEndian>().map_err(|e| ProtocolDeserializeError(String::from(e.description())))
}

pub fn de_array(bytes: Vec<u8>) -> ProtocolDeserializeResult<Vec<u8>> {
    // TODO
}

