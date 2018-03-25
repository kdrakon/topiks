extern crate byteorder;

use kafka_protocol::protocol_response::*;
use kafka_protocol::protocol_serializable::*;

pub struct DeleteTopicsResponse {
    pub throttle_time_ms: i32,
    pub topic_error_codes: Vec<TopicErrorCode>
}

pub struct TopicErrorCode {
    topic: String,
    error_code: i16
}

impl ProtocolDeserializable<Response<DeleteTopicsResponse>> for Vec<u8> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<Response<DeleteTopicsResponse>> {
        let fields: ProtocolDeserializeResult<(ResponseHeader, DeleteTopicsResponse)> =
            self[0..4].to_vec().into_protocol_type().and_then(|header| {
                self[4..].to_vec().into_protocol_type().map(|response_message| {
                    (header, response_message)
                })
            });

        fields.map(|(header, response_message)| {
            Response {
                header,
                response_message,
            }
        })
    }
}

impl ProtocolDeserializable<DeleteTopicsResponse> for Vec<u8> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<DeleteTopicsResponse> {
        unimplemented!()
    }
}

impl ProtocolDeserializable<TopicErrorCode> for Vec<u8> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<TopicErrorCode> {
        unimplemented!()
    }
}