extern crate byteorder;

use std::io::*;

use kafka_protocol::protocol_serializable::ProtocolSerializeResult;
use kafka_protocol::protocol_serializable::*;

use self::byteorder::{BigEndian, WriteBytesExt};
use self::ProtocolPrimitives::*;

/// Primitive types supported by Kafka protocol.
/// Primarily wrapped for convenience with ProtocolSerializable.
#[derive(Clone)]
pub enum ProtocolPrimitives {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Boolean(bool),
}

impl ProtocolPrimitives {
    // indicates null length
    pub fn null_bytes() -> ProtocolPrimitives {
        I32(-1)
    }
    pub fn null_string() -> ProtocolPrimitives {
        I16(-1)
    }
}

impl ProtocolSerializable for String {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        I16(self.len() as i16)
            .into_protocol_bytes()
            .and_then(|mut string_size| {
                let mut string_bytes = self.as_bytes().to_vec();
                string_size.append(&mut string_bytes);
                Ok(string_size)
            })
    }
}

impl ProtocolSerializable for Option<String> {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        match self {
            Some(string) => string.into_protocol_bytes(),
            None => ProtocolPrimitives::null_string().into_protocol_bytes(),
        }
    }
}

impl ProtocolSerializable for ProtocolPrimitives {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let mut payload: Vec<u8> = vec![];
        let serialized = match self {
            I8(i) => payload.write_i8(i),
            I16(i) => payload.write_i16::<BigEndian>(i),
            I32(i) => payload.write_i32::<BigEndian>(i),
            I64(i) => payload.write_i64::<BigEndian>(i),
            Boolean(b) => payload.write_i8(if b { 1 } else { 0 }),
        };
        serialized.map(|_| payload)
    }
}

impl<T> ProtocolSerializable for Vec<T>
where
    T: ProtocolSerializable,
{
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let array_length = I32(self.len() as i32).into_protocol_bytes();

        let sequenced = self
            .into_iter()
            .map(|t| t.into_protocol_bytes())
            .collect::<Result<Vec<Vec<u8>>>>();

        let array_in_bytes = sequenced.map(|ref s| {
            s.into_iter()
                .fold(vec![] as Vec<u8>, |mut acc: Vec<u8>, v| {
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

impl<T> ProtocolSerializable for Option<Vec<T>>
where
    T: ProtocolSerializable,
{
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        match self {
            Some(array) => array.into_protocol_bytes(),
            None => ProtocolPrimitives::null_bytes().into_protocol_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_array_and_primitives() {
        assert_eq!(
            vec![0, 0, 0, 3, 1, 2, 3],
            vec![I8(1), I8(2), I8(3)].into_protocol_bytes().unwrap()
        );
        assert_eq!(
            vec![0, 0, 0, 3, 0, 1, 0, 2, 0, 3],
            vec![I16(1), I16(2), I16(3)].into_protocol_bytes().unwrap()
        );
        assert_eq!(
            vec![0, 0, 0, 1, 0, 0, 0, 255],
            vec![I32(255)].into_protocol_bytes().unwrap()
        );
        assert_eq!(
            vec![0, 0, 0, 1, 0, 1, 0, 0],
            vec![I32(65536)].into_protocol_bytes().unwrap()
        );

        assert_eq!(
            vec![0, 0, 0, 1, 1],
            vec![Boolean(true)].into_protocol_bytes().unwrap()
        );
        assert_eq!(
            vec![0, 0, 0, 2, 0, 1],
            vec![Boolean(false), Boolean(true)]
                .into_protocol_bytes()
                .unwrap()
        );

        // array of 2, string of 3, 3 characters, string of 3, 3 characters
        assert_eq!(
            vec![0, 0, 0, 2, 0, 3, 102, 111, 111, 0, 3, 98, 97, 114],
            vec![String::from("foo"), String::from("bar")]
                .into_protocol_bytes()
                .unwrap()
        );
    }
}
