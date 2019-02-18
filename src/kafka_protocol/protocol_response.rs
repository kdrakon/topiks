extern crate byteorder;

use kafka_protocol::protocol_serializable::*;

/// Top-level response which can be sent from a Kafka broker.
///
#[derive(Debug)]
pub struct Response<T> {
    pub header: ResponseHeader,
    pub response_message: T,
}

impl<T> ProtocolDeserializable<Response<T>> for Vec<u8>
where
    Vec<u8>: ProtocolDeserializable<T>,
{
    fn into_protocol_type(self) -> ProtocolDeserializeResult<Response<T>> {
        ProtocolDeserializable::<ResponseHeader>::into_protocol_type(self[0..4].to_vec()).and_then(
            |header| {
                ProtocolDeserializable::<T>::into_protocol_type(self[4..].to_vec()).map(
                    |response_message| Response {
                        header,
                        response_message,
                    },
                )
            },
        )
    }
}

/// Header information for a Response
///
#[derive(Debug)]
pub struct ResponseHeader {
    pub correlation_id: i32,
}

impl ProtocolDeserializable<ResponseHeader> for Vec<u8> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<ResponseHeader> {
        de_i32(self).map(|correlation_id| ResponseHeader { correlation_id })
    }
}
