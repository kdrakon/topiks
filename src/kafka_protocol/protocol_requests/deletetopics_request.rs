use kafka_protocol::protocol_primitives::*;
use kafka_protocol::protocol_primitives::ProtocolPrimitives::*;
use kafka_protocol::protocol_serializable::*;
use kafka_protocol::protocol_serializable::ProtocolSerializeResult;

#[derive(Clone)]
pub struct DeleteTopicsRequest {
    pub topics: Vec<String>,
    pub timeout: i32,
}

impl ProtocolSerializable for DeleteTopicsRequest {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let topics = self.topics;
        let timeout = I32(self.timeout);
        topics.into_protocol_bytes().and_then(|mut t| {
            timeout.into_protocol_bytes().map(|ref mut a| {
                t.append(a);
                t
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use kafka_protocol::protocol_requests::deletetopics_request::*;

    proptest! {
        #[test]
        fn verify_serde_for_deletetopics_request(ref topic_a in ".*", ref topic_b in ".*") {
            let request = DeleteTopicsRequest {
                topics: vec![topic_a.clone(), topic_b.clone()],
                timeout: 42
            };
            match request.into_protocol_bytes() {
                Ok(bytes) => (),
                Err(e) => panic!(e)
            };
        }
    }
}