use kafka_protocol::protocol_serializable::*;

/// Version 2
#[derive(Debug)]
pub struct ListOffsetsResponse {
    pub throttle_time_ms: i32,
    pub responses: Vec<Response>,
}

#[derive(Debug)]
pub struct Response {
    pub topic: String,
    pub partition_responses: Vec<PartitionResponse>,
}

#[derive(Debug, Clone)]
pub struct PartitionResponse {
    pub partition: i32,
    pub error_code: i16,
    pub timestamp: i64,
    pub offset: i64,
}

impl ProtocolDeserializable<ListOffsetsResponse> for Vec<u8> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<ListOffsetsResponse> {
        de_i32(self[0..=3].to_vec()).and_then(|throttle_time_ms| {
            de_array(self[4..].to_vec(), deserialize_response).map(|(responses, _bytes)| {
                ListOffsetsResponse {
                    throttle_time_ms,
                    responses,
                }
            })
        })
    }
}

fn deserialize_response(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicSize<Response>> {
    de_string(bytes).and_then(|(topic, bytes)| {
        de_array(bytes, deserialize_partition_response).and_then(|(partition_responses, bytes)| {
            topic
                .map(|topic| {
                    Ok((
                        Response {
                            topic,
                            partition_responses,
                        },
                        bytes,
                    ))
                })
                .unwrap_or(Err(DeserializeError::of("Topic name undefined")))
        })
    })
}

fn deserialize_partition_response(
    bytes: Vec<u8>,
) -> ProtocolDeserializeResult<DynamicSize<PartitionResponse>> {
    de_i32(bytes[0..=3].to_vec()).and_then(|partition| {
        de_i16(bytes[4..=5].to_vec()).and_then(|error_code| {
            de_i64(bytes[6..=13].to_vec()).and_then(|timestamp| {
                de_i64(bytes[14..=21].to_vec()).map(|offset| {
                    let remaining_bytes = if bytes.len() > 22 {
                        bytes[22..].to_vec()
                    } else {
                        vec![]
                    };
                    (
                        PartitionResponse {
                            partition,
                            error_code,
                            timestamp,
                            offset,
                        },
                        remaining_bytes,
                    )
                })
            })
        })
    })
}
