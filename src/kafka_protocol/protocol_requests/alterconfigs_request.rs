use kafka_protocol::api_verification::KafkaApiVersioned;
use kafka_protocol::protocol_primitives::*;
use kafka_protocol::protocol_request::Request;
use kafka_protocol::protocol_requests;
use kafka_protocol::protocol_response::Response;
use kafka_protocol::protocol_responses::alterconfigs_response::AlterConfigsResponse;
use kafka_protocol::protocol_serializable::*;
use state::StateFNError;
use api_client::ApiClientTrait;
use api_client::ApiClient;
use api_client::TcpRequestError;

#[derive(Clone, Debug)]
pub struct AlterConfigsRequest {
    pub resources: Vec<Resource>,
    pub validate_only: bool,
}

#[derive(Clone, Debug)]
pub struct Resource {
    pub resource_type: i8,
    pub resource_name: String,
    pub config_entries: Vec<ConfigEntry>,
}

#[derive(Clone, Debug)]
pub struct ConfigEntry {
    pub config_name: String,
    pub config_value: Option<String>,
}

impl KafkaApiVersioned for AlterConfigsRequest {
    fn api_key() -> i16 { 33 }
    fn version() -> i16 { 0 }
}

impl ProtocolSerializable for AlterConfigsRequest {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let resources = self.resources;
        let validate_only = self.validate_only;
        resources.into_protocol_bytes().and_then(|mut resources| {
            ProtocolPrimitives::Boolean(validate_only).into_protocol_bytes().map(|ref mut validate_only| {
                resources.append(validate_only);
                resources
            })
        })
    }
}

impl ProtocolSerializable for Resource {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let resource_type = self.resource_type;
        let resource_name = self.resource_name;
        let config_entries = self.config_entries;
        ProtocolPrimitives::I8(resource_type).into_protocol_bytes().and_then(|mut resource_type| {
            resource_name.into_protocol_bytes().and_then(|ref mut resource_name| {
                config_entries.into_protocol_bytes().map(|ref mut config_entries| {
                    resource_type.append(resource_name);
                    resource_type.append(config_entries);
                    resource_type
                })
            })
        })
    }
}

impl ProtocolSerializable for ConfigEntry {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let config_name = self.config_name;
        let config_value = self.config_value;
        config_name.into_protocol_bytes().and_then(|mut config_name| {
            config_value.into_protocol_bytes().map(|ref mut config_value| {
                config_name.append(config_value);
                config_name
            })
        })
    }
}
