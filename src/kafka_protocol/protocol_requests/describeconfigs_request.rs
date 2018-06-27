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
    pub config_names: Option<Vec<String>>
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
        let resources = self.resources;
        let include_synonyms = self.include_synonyms;
        resources.into_protocol_bytes().and_then(|mut resources| {
            ProtocolPrimitives::Boolean(include_synonyms).into_protocol_bytes().map(|ref mut include_synonyms|{
                resources.append(include_synonyms);
                resources
            })
        })
    }
}

impl ProtocolSerializable for Resource {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let resource_type = ProtocolPrimitives::I8(self.resource_type);
        let resource_name = self.resource_name;
        let config_names = self.config_names;
        resource_type.into_protocol_bytes().and_then(|mut resource_type|{
            resource_name.clone().into_protocol_bytes().and_then(|ref mut resource_name|{
                config_names.into_protocol_bytes().map(|ref mut config_names|{
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
            Resource { resource_type: 1, resource_name: name_a.clone(), config_names: Some(vec![String::from("foo")]) },
            Resource { resource_type: 1, resource_name: name_b, config_names: Some(vec![String::from("bar")]) },
            Resource { resource_type: 1, resource_name: name_a, config_names: None }
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