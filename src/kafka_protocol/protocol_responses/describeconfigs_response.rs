use kafka_protocol::protocol_response::Response;
use kafka_protocol::protocol_serializable::*;
use kafka_protocol::protocol_response::ResponseHeader;

pub struct DescribeConfigsResponse {
    pub throttle_time_ms: i32,
    pub resources: Vec<Resource>,
}

pub struct Resource {
    error_code: i16,
    error_message: Option<String>,
    resource_type: i8,
    resource_name: String,
    config_entries: Vec<ConfigEntry>,
}

pub struct ConfigEntry {
    config_name: String,
    config_value: Option<String>,
    read_only: bool,
    config_source: i8,
    is_sensitive: bool,
    config_synonyms: Vec<ConfigSynonym>,
}

pub struct ConfigSynonym {
    config_name: String,
    config_value: Option<String>,
    config_source: i8,
}

impl ProtocolDeserializable<DescribeConfigsResponse> for Vec<u8> {
    fn into_protocol_type(self) -> Result<DescribeConfigsResponse, DeserializeError> {
        de_i32(self[0..=3].to_vec()).and_then(|throttle_time_ms| {
            de_array(self[4..].to_vec(), deserialize_resource).map(|(resources, remaining_bytes)| {
                if !remaining_bytes.is_empty() { panic!("Unexpected bytes deserializing DescribeConfigsResponse") }
                DescribeConfigsResponse { throttle_time_ms, resources }
            })
        })
    }
}

fn deserialize_resource(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicType<Resource>> {
    unimplemented!()
}