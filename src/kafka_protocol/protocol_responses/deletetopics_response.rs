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
        de_i32(self[0..4].to_vec()).and_then(|throttle_time_ms| {

            let topic_error_codes: ProtocolDeserializeResult<DynamicType<Vec<TopicErrorCode>>> =
                de_array(self[4..].to_vec(), deserialize_topic_error_codes);

            topic_error_codes.map(|(topic_error_codes, remaining_bytes)|{
                if !remaining_bytes.is_empty() { panic!("Unexpected bytes deserializing DeleteTopicsResponse") }
                DeleteTopicsResponse {
                    throttle_time_ms,
                    topic_error_codes
                }
            })
        })
    }
}

fn deserialize_topic_error_codes(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicType<TopicErrorCode>> {
    de_string(bytes).and_then(|(topic, remaining_bytes)| {
        let topic = topic.expect("Unexpected missing topic name in TopicErrorCode");
        de_i16(remaining_bytes[0..2].to_vec()).map(|error_code| {
            (
                TopicErrorCode { topic, error_code },
                remaining_bytes
            )
        })
    })
}