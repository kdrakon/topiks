extern crate byteorder;

use self::byteorder::{BigEndian, ReadBytesExt};
use std::error::Error;
use std::io::Cursor;
use std::io::Result as IOResult;
use std::str::from_utf8;

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
    where F: Fn(Vec<u8>) -> ProtocolDeserializeResult<DynamicType<T>> {

    let array_size = de_i32(bytes[0..4].to_vec());
    array_size.and_then(|expected_elements| {
        let element_bytes = bytes[4..].to_vec();
        de_array_transform(element_bytes, expected_elements, deserialize_t)
    })
}

fn de_array_transform<T, F>(bytes: Vec<u8>, elements: i32, deserialize_t: F) -> ProtocolDeserializeResult<DynamicType<Vec<T>>>
    where F: Fn(Vec<u8>) -> ProtocolDeserializeResult<DynamicType<T>> {

    if elements <= 0 {
        Ok((vec![] as Vec<T>, bytes))
    } else {
        deserialize_t(bytes).and_then(|(t, leftover_bytes)|{
            match de_array_transform(leftover_bytes, elements - 1, deserialize_t) {
                Ok((mut next_ts, leftover_bytes)) => {
                    let mut ts = vec![t];
                    ts.append(&mut next_ts);
                    Ok((ts, leftover_bytes))
                },
                err @ Err(_) => err
            }
        })
    }
}

pub fn de_string(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicType<Option<String>>> {
    de_i16(bytes[0..2].to_vec()).and_then(|byte_length|{
        match byte_length {
            -1 => Ok((None, bytes[2..].to_vec())),
            _ => {
                let end_index = (byte_length as usize) + 2;
                let remaining_bytes = bytes[end_index..].to_vec();
                let string_bytes = &bytes[2..end_index];

                match from_utf8(string_bytes) {
                    Ok(string) => Ok((Some(String::from(string)), remaining_bytes)),
                    _ => Err(DeserializeError::of(String::from("Failed to deserialize string")))
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {

    use kafka_protocol::protocol_primitives::*;
    use kafka_protocol::protocol_primitives::ProtocolPrimitives::*;
    use super::*;

    proptest! {
        #[test]
        fn verify_de_string(ref s in ".*") {

            let bytes = s.clone().into_protocol_bytes().unwrap();
            match de_string(bytes) {
                Ok((Some(string), remaining_bytes)) => {
                    assert!(remaining_bytes.is_empty());
                    assert_eq!(s.clone(), string);
                },
                _ => panic!("test failed")
            }

            // verify null string
            let mut bytes = I16(-1).into_protocol_bytes().unwrap();
            bytes.append(&mut vec![40, 41, 42]);
            match de_string(bytes) {
                Ok((None, remaining_bytes)) => {
                    assert_eq!(remaining_bytes, vec![40, 41, 42]);
                }
                _ => panic!("test failed")
            }
        }
    }

    proptest! {
        #[test]
        fn verify_de_array(ref a in ".*", ref b in ".*", ref c in ".*") {

            let array = ProtocolArray::of(vec![a.clone(), b.clone(), c.clone()]);
            let bytes = array.into_protocol_bytes().unwrap();
            let result =
                de_array(bytes, |element| {
                    de_string(element).map(|(opt_string, remaining_bytes)| {
                        (opt_string.expect("should be deserializable string"), remaining_bytes)
                    })
                });

            match result {
                Ok((strings, remaining_bytes)) => {
                    assert_eq!(3, strings.len());
                    assert_eq!(vec![a.clone(), b.clone(), c.clone()], strings);
                    assert_eq!(0, remaining_bytes.len());
                },
                _ => panic!("test failed")
            }
        }
    }
}
