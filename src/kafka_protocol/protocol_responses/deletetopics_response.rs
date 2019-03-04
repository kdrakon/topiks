extern crate byteorder;

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use kafka_protocol::protocol_serializable::*;

pub struct DeleteTopicsResponse {
    pub throttle_time_ms: i32,
    pub topic_error_codes: Vec<TopicErrorCode>,
}

#[derive(Debug)]
pub struct TopicErrorCode {
    pub topic: String,
    pub error_code: i16,
}

impl Display for TopicErrorCode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Topic Error for {}: {}", self.topic, self.error_code)
    }
}

impl ProtocolDeserializable<DeleteTopicsResponse> for Vec<u8> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<DeleteTopicsResponse> {
        de_i32(self[0..4].to_vec()).and_then(|throttle_time_ms| {
            let topic_error_codes: ProtocolDeserializeResult<DynamicSize<Vec<TopicErrorCode>>> =
                de_array(self[4..].to_vec(), deserialize_topic_error_codes);

            topic_error_codes.map(|(topic_error_codes, remaining_bytes)| {
                if !remaining_bytes.is_empty() {
                    panic!("Unexpected bytes deserializing DeleteTopicsResponse")
                }
                DeleteTopicsResponse { throttle_time_ms, topic_error_codes }
            })
        })
    }
}

fn deserialize_topic_error_codes(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicSize<TopicErrorCode>> {
    de_string(bytes).and_then(|(topic, remaining_bytes)| {
        let topic = topic.expect("Unexpected missing topic name in TopicErrorCode");
        de_i16(remaining_bytes[0..2].to_vec()).map(|error_code| (TopicErrorCode { topic, error_code }, remaining_bytes[2..].to_vec()))
    })
}
