extern crate byteorder;

use self::byteorder::{BigEndian, WriteBytesExt};
use self::ProtocolPrimitives::*;
use std::string::ToString;

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
struct ProtocolArray<T> {
    array: Vec<T>
}

/// If implemented, a struct/enum can be sent on the wire to a
/// Kafka broker.
///
trait ProtocolSerializable {
    fn to_protocol_bytes(&self) -> ProtocolSerializeResult;
}

pub type ProtocolSerializeResult<'a> = Result<Vec<u8>, ProtocolSerializeError<'a>>;

pub struct ProtocolSerializeError<'a>(&'a str);

impl ProtocolSerializable for String {
    fn to_protocol_bytes(&self) -> ProtocolSerializeResult {
        let mut payload: Vec<u8> = vec![];
        let string_size: i16 = self.len() as i16;
        let mut string_bytes: Vec<u8> = self.as_bytes().to_vec();

        // i16 String Size + String byte array
        payload.write_i16::<BigEndian>(string_size);
        payload.append(&mut string_bytes);
        Err(ProtocolSerializeError("TODO")) //payload
    }
}

impl ProtocolSerializable for ProtocolPrimitives {
    fn to_protocol_bytes(&self) -> ProtocolSerializeResult {
        let mut payload: Vec<u8> = vec![];
        let serialization =
            match self {
                &I8(i) => payload.write_i8(i),
                &I16(i) => payload.write_i16::<BigEndian>(i),
                &I32(i) => payload.write_i32::<BigEndian>(i)
            };
        serialization
            .map_err(|e| ProtocolSerializeError(&e.to_string()))
            .map(|_| payload)
    }
}

impl<T> ProtocolSerializable for ProtocolArray<T>
    where T: ProtocolSerializable {
    fn to_protocol_bytes(&self) -> ProtocolSerializeResult {
        // TODO size + byte array
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
    fn to_protocol_bytes(&self) -> ProtocolSerializeResult {
        let header_and_request =
            self.header.to_protocol_bytes().and_then(|ref mut header|
                match self.request_message.to_protocol_bytes() {
                    Ok(ref mut request_message) => {
                        header.append(request_message);
                        Ok(header.clone())
                    },
                    Err(e) => Err(e)
                }
            );

        header_and_request.and_then(|ref mut hr|
            match I32(hr.len() as i32).to_protocol_bytes() {
                Ok(ref mut message_size) => {
                    message_size.append(hr);
                    Ok(message_size.clone())
                },
                Err(e) => Err(e)
            }
        )
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
    fn to_protocol_bytes(&self) -> ProtocolSerializeResult {
        I16(self.api_key).to_protocol_bytes().and_then(|ref mut api_key| {
            I16(self.api_version).to_protocol_bytes().and_then(|ref mut api_version| {
                I32(self.correlation_id).to_protocol_bytes().and_then(|ref mut correlation_id| {
                    self.client_id.to_protocol_bytes().and_then(|ref mut client_id| {
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

pub enum RequestMessage {
    MetadataRequest { topics: Vec<String> }
}

impl ProtocolSerializable for RequestMessage {
    fn to_protocol_bytes(&self) -> ProtocolSerializeResult {
        unimplemented!()
    }
}