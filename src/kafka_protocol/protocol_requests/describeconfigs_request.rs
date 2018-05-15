use kafka_protocol::protocol_primitives::ProtocolArray;
use kafka_protocol::protocol_primitives::ProtocolPrimitives;
use kafka_protocol::protocol_serializable::ProtocolSerializable;
use kafka_protocol::protocol_serializable::ProtocolSerializeResult;

pub struct DescribeConfigsRequest {
    pub resources: Vec<Resource>,
    pub include_synonyms: bool
}

#[derive(Clone)]
pub struct Resource {
    pub resource_type: i8,
    pub resource_name: String,
    pub config_names: String
}

impl ProtocolSerializable for DescribeConfigsRequest {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let resources = ProtocolArray::of(self.resources.clone()).into_protocol_bytes();
        resources.and_then(|mut resources| {
            ProtocolPrimitives::Boolean(self.include_synonyms).into_protocol_bytes().map(|ref mut include_synonyms|{
                resources.append(include_synonyms);
                resources
            })
        })
    }
}

impl ProtocolSerializable for Resource {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        ProtocolPrimitives::I8(self.resource_type).into_protocol_bytes().and_then(|mut resource_type|{
            self.resource_name.clone().into_protocol_bytes().and_then(|ref mut resource_name|{
                self.config_names.into_protocol_bytes().map(|ref mut config_names|{
                    resource_type.append(resource_name);
                    resource_type.append(config_names);
                    resource_type
                })
            })
        })
    }
}