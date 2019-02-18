use kafka_protocol::protocol_serializable::*;

#[derive(Clone, Debug)]
pub struct DescribeConfigsResponse {
    pub throttle_time_ms: i32,
    pub resources: Vec<Resource>,
}

#[derive(Clone, Debug)]
pub struct Resource {
    pub error_code: i16,
    pub error_message: Option<String>,
    pub resource_type: i8,
    pub resource_name: String,
    pub config_entries: Vec<ConfigEntry>,
}

#[derive(Clone, Debug)]
pub struct ConfigEntry {
    pub config_name: String,
    pub config_value: Option<String>,
    pub read_only: bool,
    pub config_source: i8,
    pub is_sensitive: bool,
    pub config_synonyms: Vec<ConfigSynonym>,
}

#[derive(Clone, Debug)]
pub struct ConfigSynonym {
    pub config_name: String,
    pub config_value: Option<String>,
    pub config_source: i8,
}

pub enum ConfigSource {
    UnknownConfig = 0,
    TopicConfig = 1,
    DynamicBrokerConfig = 2,
    DynamicDefaultBrokerConfig = 3,
    StaticBrokerConfig = 4,
    DefaultConfig = 5,
}

impl ProtocolDeserializable<DescribeConfigsResponse> for Vec<u8> {
    fn into_protocol_type(self) -> Result<DescribeConfigsResponse, DeserializeError> {
        de_i32(self[0..=3].to_vec()).and_then(|throttle_time_ms| {
            de_array(self[4..].to_vec(), deserialize_resource).map(
                |(resources, remaining_bytes)| {
                    if !remaining_bytes.is_empty() {
                        panic!("Unexpected bytes deserializing DescribeConfigsResponse")
                    }
                    DescribeConfigsResponse {
                        throttle_time_ms,
                        resources,
                    }
                },
            )
        })
    }
}

fn deserialize_resource(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicSize<Resource>> {
    de_i16(bytes[0..=1].to_vec()).and_then(|error_code| {
        de_string(bytes[2..].to_vec()).and_then(|(error_message, bytes)| {
            Ok((bytes[0] as i8, bytes[1..].to_vec())).and_then(|(resource_type, bytes)| {
                de_string(bytes).and_then(|(resource_name, bytes)| {
                    de_array(bytes, deserialize_config_entry).map(|(config_entries, bytes)| {
                        (
                            Resource {
                                error_code,
                                error_message,
                                resource_type,
                                resource_name: resource_name
                                    .expect("Unexpected undefined resource name"),
                                config_entries,
                            },
                            bytes,
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
            Ok((read_only, config_source, is_sensitive, bytes[3..].to_vec())).and_then(
                |(read_only, config_source, is_sensitive, bytes)| {
                    de_array(bytes, deserialize_config_synonym).map(|(config_synonyms, bytes)| {
                        (
                            ConfigEntry {
                                config_name: config_name.expect("Unexpected undefined config name"),
                                config_value,
                                read_only,
                                config_source,
                                is_sensitive,
                                config_synonyms,
                            },
                            bytes,
                        )
                    })
                },
            )
        })
    })
}

fn deserialize_config_synonym(
    bytes: Vec<u8>,
) -> ProtocolDeserializeResult<DynamicSize<ConfigSynonym>> {
    de_string(bytes).and_then(|(config_name, bytes)| {
        de_string(bytes).map(|(config_value, bytes)| {
            (
                ConfigSynonym {
                    config_name: config_name.expect("Unexpected undefined config name"),
                    config_value,
                    config_source: bytes[0] as i8,
                },
                bytes[1..].to_vec(),
            )
        })
    })
}
