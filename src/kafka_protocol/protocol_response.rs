extern crate byteorder;

use self::byteorder::{BigEndian, ReadBytesExt};
use std::error::Error;
use std::io::*;
use kafka_protocol::protocol_primitives::*;
use kafka_protocol::protocol_serializable::*;

/// Top-level response which can be sent from a Kafka broker.
///
pub struct Response<T> {
    pub header: ResponseHeader,
    pub response_message: T,
}

/// Header information for a Response
///
pub struct ResponseHeader {
    pub correlation_id: i32
}

impl ProtocolDeserializable<ResponseHeader> for Vec<u8> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<ResponseHeader> {
        de_i32(self).map(|correlation_id| {
            ResponseHeader { correlation_id }
        })
    }
}