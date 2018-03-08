extern crate byteorder;

use super::protocol_response::*;
use self::byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use std::io::Result;
use std::error::Error;

/// If implemented, a struct/enum can be sent on the wire to a
/// Kafka broker.
///
pub trait ProtocolSerializable {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult;
}

pub type ProtocolSerializeResult = Result<Vec<u8>>;

/// If implemented, a Vec<u8> can be read from a Kafka broker
/// into a type T
///
pub trait ProtocolDeserializable<T> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<T>;
}

pub type ProtocolDeserializeResult<T> = Result<T>;

// Deserializer Functions
pub fn de_i32(bytes: Vec<u8>) -> ProtocolDeserializeResult<i32> {
    Cursor::new(bytes).read_i32::<BigEndian>()
}

pub fn de_array(bytes: Vec<u8>) -> ProtocolDeserializeResult<Vec<u8>> {
//    let array_size = de_i32(bytes[0..3]);
    unimplemented!()
}

