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
    pub resource_name: Option<String>,
    pub config_names: Option<String>
}

pub enum ResourceTypes {
    Unknown = 0,
    Any = 1,
    Topic = 2,
    Group = 3,
    Cluster = 4,
    Broker = 5,
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

#[cfg(test)]
mod tests {

    use kafka_protocol::protocol_requests::describeconfigs_request::*;

    fn verify_serde_for_describeconfigs_request(name_a: String, name_b: String) {
        let resources = vec![
            Resource { resource_type: 1, resource_name: Some(name_a), config_names: Some(String::from("foo")) },
            Resource { resource_type: 1, resource_name: Some(name_b), config_names: Some(String::from("bar")) },
            Resource { resource_type: 1, resource_name: None, config_names: None }
        ];
        let request = DescribeConfigsRequest { resources, include_synonyms: false };
        match request.into_protocol_bytes() {
            Ok(bytes) => (),
            Err(e) => panic!(e)
        }
    }

    proptest! {
        #[test]
        fn verify_serde_for_describeconfigs_request_props(ref name_a in ".*", ref name_b in ".*") {
            verify_serde_for_describeconfigs_request(name_a.to_string(), name_b.to_string())
        }
    }
}