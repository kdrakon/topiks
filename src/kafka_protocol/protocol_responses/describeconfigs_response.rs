use kafka_protocol::protocol_response::Response;
use kafka_protocol::protocol_response::ResponseHeader;
use kafka_protocol::protocol_serializable::*;

pub struct DescribeConfigsResponse {
    pub throttle_time_ms: i32,
    pub resources: Vec<Resource>,
}

#[derive(Clone)]
pub struct Resource {
    error_code: i16,
    error_message: Option<String>,
    resource_type: i8,
    resource_name: String,
    config_entries: Vec<ConfigEntry>,
}

#[derive(Clone)]
pub struct ConfigEntry {
    config_name: String,
    config_value: Option<String>,
    read_only: bool,
    config_source: i8,
    is_sensitive: bool,
    config_synonyms: Vec<ConfigSynonym>,
}

#[derive(Clone)]
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

fn deserialize_resource(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicSize<Resource>> {
    de_i16(bytes[0..=1].to_vec()).and_then(|error_code| {
        de_string(bytes[1..].to_vec()).and_then(|(error_message, bytes)| {
            Ok((bytes[0] as i8, bytes[1..].to_vec())).and_then(|(resource_type, bytes)| {
                de_string(bytes).and_then(|(resource_name, bytes)| {
                    de_array(bytes, deserialize_config_entry).map(|(config_entries, bytes)| {
                        (
                            Resource { error_code, error_message, resource_type, resource_name: resource_name.expect("Unexpected undefined resource name"), config_entries },
                            bytes
                        )
                    })
                })
            })
        })
    })
}

fn deserialize_config_entry(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicSize<ConfigEntry>> {
    de_string(bytes).and_then(|(config_name, bytes)| {
        de_string(bytes).and_then(|(config_value, bytes)| {
            let read_only = bytes[0] == 1;
            let config_source = bytes[1] as i8;
            let is_sensitive = bytes[2] == 1;
            Ok((read_only, config_source, is_sensitive, bytes[3..].to_vec())).and_then(|(read_only, config_source, is_sensitive, bytes)| {
                de_array(bytes, deserialize_config_synonym).map(|(config_synonyms, bytes)| {
                    (
                        ConfigEntry { config_name: config_name.expect("Unexpected undefined config name"), config_value, read_only, config_source, is_sensitive, config_synonyms },
                        bytes
                    )
                })
            })
        })
    })
}

fn deserialize_config_synonym(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicSize<ConfigSynonym>> {
    de_string(bytes).and_then(|(config_name, bytes)| {
        de_string(bytes).map(|(config_value, bytes)| {
            (
                ConfigSynonym { config_name: config_name.expect("Unexpected undefined config name"), config_value, config_source: bytes[0] as i8 },
                bytes[1..].to_vec()
            )
        })
    })
}