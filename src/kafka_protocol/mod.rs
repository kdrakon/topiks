extern crate byteorder;

use self::byteorder::{BigEndian, WriteBytesExt};
use self::ProtocolPrimitives::*;
use self::RequestMessage::*;
use std::error::Error;

/// Primitive types supported by Kafka protocol.
/// Primarily wrapped for convenience with ProtocolSerializable.
///
pub enum ProtocolPrimitives {
    I8(i8),
    I16(i16),
    I32(i32),
}

/// Wrapper to a vector for convenience with
/// ProtocolSerializable
///
pub struct ProtocolArray<T> {
    array: Vec<T>
}

/// If implemented, a struct/enum can be sent on the wire to a
/// Kafka broker.
///
trait ProtocolSerializable {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult;
}

pub type ProtocolSerializeResult = Result<Vec<u8>, ProtocolSerializeError>;
pub struct ProtocolSerializeError(String);

impl ProtocolSerializable for String {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        I16(self.len() as i16).into_protocol_bytes().and_then(|mut string_size|{
            let mut string_bytes = self.as_bytes().to_vec();
            string_size.append(&mut string_bytes);
            Ok(string_size.clone())
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
                I32(i) => payload.write_i32::<BigEndian>(i)
            };
        serialized
            .map_err(|e| ProtocolSerializeError(String::from(e.description())))
            .map(|_| payload)
    }
}

impl<T> ProtocolSerializable for ProtocolArray<T>
    where T: ProtocolSerializable {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let sequenced =
            self.array.into_iter().map(|t| {
                t.into_protocol_bytes()
            }).collect::<Result<Vec<Vec<u8>>, ProtocolSerializeError>>();

        let array_in_bytes =
            sequenced.map(|ref s| {
                s.iter().fold(vec![], |mut acc: Vec<u8>, v| {
                    acc.append(&mut v.clone());
                    acc
                })
            });

        unimplemented!()
    }
}

/// Top-level request which can be sent to a Kafka broker.
///
pub struct Request {
    pub header: RequestHeader,
    pub request_message: RequestMessage,
}

impl ProtocolSerializable for Request {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let header = self.header;
        let request_message = self.request_message;
        let header_and_request =
            header.into_protocol_bytes().and_then(|mut h|
                request_message.into_protocol_bytes().map(|ref mut rm| {
                    h.append(rm);
                    h
                })
            );

        header_and_request.and_then(|ref mut hr|
            I32(hr.len() as i32).into_protocol_bytes().map(|mut message_size| {
                message_size.append(hr);
                message_size
            }))
    }
}

/// Header information for a Request
///
pub struct RequestHeader {
    pub api_key: i16,
    pub api_version: i16,
    pub correlation_id: i32,
    pub client_id: String,
}

impl ProtocolSerializable for RequestHeader {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        I16(self.api_key).into_protocol_bytes().and_then(|mut api_key| {
            I16(self.api_version).into_protocol_bytes().and_then(|ref mut api_version| {
                I32(self.correlation_id).into_protocol_bytes().and_then(|ref mut correlation_id| {
                    self.client_id.into_protocol_bytes().and_then(|ref mut client_id| {
                        api_key.append(api_version);
                        api_key.append(correlation_id);
                        api_key.append(client_id);
                        Ok(api_key.clone())
                    })
                })
            })
        })
    }
}

/// Enum of requests that can be sent to the Kafka API
///
pub enum RequestMessage {
    MetadataRequest { topics: ProtocolArray<String> }
}

impl ProtocolSerializable for RequestMessage {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        match self {
            MetadataRequest { ref topics } => unimplemented!()
        }
        unimplemented!()
    }
}

impl RequestMessage {
    fn serialize_metadata_request(topics: &ProtocolArray<String>) -> ProtocolSerializeResult {
        // TODO
        unimplemented!()
    }
}