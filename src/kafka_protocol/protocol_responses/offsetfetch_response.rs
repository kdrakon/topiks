use kafka_protocol::protocol_serializable::*;

/// Version 3
#[derive(Debug)]
pub struct OffsetFetchResponse {
    pub throttle_time_ms: i32,
    pub responses: Vec<Response>,
    pub error_code: i16,
}

#[derive(Debug)]
pub struct Response {
    pub topic: String,
    pub partition_responses: Vec<PartitionResponse>,
}

#[derive(Debug, Clone)]
pub struct PartitionResponse {
    pub partition: i32,
    pub offset: i64,
    pub metadata: Option<String>,
    pub error_code: i16,
}

impl ProtocolDeserializable<OffsetFetchResponse> for Vec<u8> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<OffsetFetchResponse> {
        de_i32(self[0..=3].to_vec()).and_then(|throttle_time_ms| {
            de_array(self[4..].to_vec(), deserialize_responses).and_then(|(responses, bytes)| {
                de_i16(bytes[0..=1].to_vec()).map(|error_code| OffsetFetchResponse { throttle_time_ms, responses, error_code })
            })
        })
    }
}

fn deserialize_responses(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicSize<Response>> {
    de_string(bytes).and_then(|(topic, bytes)| {
        topic.ok_or_else(move || DeserializeError::of("Expected topic name")).and_then(|topic| {
            de_array(bytes, deserialize_partition_responses).map(|(partition_responses, bytes)| (Response { topic, partition_responses }, bytes))
        })
    })
}

fn deserialize_partition_responses(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicSize<PartitionResponse>> {
    de_i32(bytes[0..=3].to_vec()).and_then(|partition| {
        de_i64(bytes[4..=11].to_vec()).and_then(|offset| {
            de_string(bytes[12..].to_vec()).and_then(|(metadata, bytes)| {
                de_i16(bytes[0..=1].to_vec()).map(|error_code| {
                    let remaining_bytes = if bytes.len() > 2 { bytes[2..].to_vec() } else { vec![] };
                    (PartitionResponse { partition, offset, metadata, error_code }, remaining_bytes)
                })
            })
        })
    })
}
