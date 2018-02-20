extern crate byteorder;

use self::byteorder::{BigEndian, WriteBytesExt};
use self::ProtocolPrimitives::*;
use std::error::Error;
use super::protocol_serializable::*;
use super::protocol_serializable::ProtocolSerializeError;

type ProtocolSerializeResult = Result<Vec<u8>, ProtocolSerializeError>;

/// Primitive types supported by Kafka protocol.
/// Primarily wrapped for convenience with ProtocolSerializable.
///
pub enum ProtocolPrimitives {
    I8(i8),
    I16(i16),
    I32(i32),
    Boolean(bool),
}

/// Wrapper to a vector for convenience with
/// ProtocolSerializable
///
pub struct ProtocolArray<T> {
    array: Vec<T>
}

impl<T> ProtocolArray<T> {
    pub fn of(array: Vec<T>) -> ProtocolArray<T> {
        ProtocolArray { array }
    }
}

impl ProtocolSerializable for String {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        I16(self.len() as i16).into_protocol_bytes().and_then(|mut string_size| {
            let mut string_bytes = self.as_bytes().to_vec();
            string_size.append(&mut string_bytes);
            Ok(string_size)
        })
    }
}

impl ProtocolSerializable for ProtocolPrimitives {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let mut payload: Vec<u8> = vec![];
        let serialized =
            match self {
                I8(i) => payload.write_i8(i),
                I16(i) => payload.write_i16::<BigEndian>(i),
                I32(i) => payload.write_i32::<BigEndian>(i),
                Boolean(b) => payload.write_i8(if b { 1 } else { 0 })
            };
        serialized
            .map_err(|e| ProtocolSerializeError(String::from(e.description())))
            .map(|_| payload)
    }
}

impl<T> ProtocolSerializable for ProtocolArray<T>
    where T: ProtocolSerializable {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let array_length =
            I32(self.array.len() as i32).into_protocol_bytes();

        let sequenced =
            self.array.into_iter().map(|t| {
                t.into_protocol_bytes()
            }).collect::<Result<Vec<Vec<u8>>, ProtocolSerializeError>>();

        let array_in_bytes =
            sequenced.map(|ref s| {
                s.into_iter().fold(vec![] as Vec<u8>, |mut acc: Vec<u8>, v| {
                    acc.append(&mut v.clone());
                    acc
                })
            });

        array_length.and_then(|mut payload| {
            array_in_bytes.map(|ref mut aib| {
                payload.append(aib);
                payload
            })
        })
    }
}