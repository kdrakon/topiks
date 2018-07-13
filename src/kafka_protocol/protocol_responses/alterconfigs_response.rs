use kafka_protocol::protocol_serializable::*;

/// Version 0
///
#[derive(Debug)]
pub struct AlterConfigsResponse {
    pub throttle_time_ms: i32,
    pub resources: Vec<Resource>,
}

#[derive(Debug)]
pub struct Resource {
    pub error_code: i16,
    pub error_message: Option<String>,
    pub resource_type: i8,
    pub resource_name: String,
}

impl ProtocolDeserializable<AlterConfigsResponse> for Vec<u8> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<AlterConfigsResponse> {
        de_i32(self[0..=3].to_vec()).and_then(|throttle_time_ms| {
            de_array(self[4..].to_vec(), deserialize_resource).map(|(resources, bytes)| {
                AlterConfigsResponse { throttle_time_ms, resources }
            })
        })
    }
}

fn deserialize_resource(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicSize<Resource>> {
    de_i16(bytes[0..=1].to_vec()).and_then(|error_code| {
        de_string(bytes[2..].to_vec()).and_then(|(error_message, bytes)| {
            Ok((bytes[0] as i8, bytes[1..].to_vec())).and_then(|(resource_type, bytes)| {
                de_string(bytes).and_then(|(resource_name, bytes)| {
                    match resource_name {
                        None => Err(DeserializeError::of("resource_name unexpectedly null")),
                        Some(resource_name) => {
                            Ok((Resource { error_code, error_message, resource_type, resource_name }, bytes))
                        }
                    }
                })
            })
        })
    })
}

