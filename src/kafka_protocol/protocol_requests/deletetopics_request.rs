use kafka_protocol::protocol_primitives::*;
use kafka_protocol::protocol_primitives::ProtocolPrimitives::*;
use kafka_protocol::protocol_serializable::*;
use kafka_protocol::protocol_serializable::ProtocolSerializeResult;

pub struct DeleteTopicsRequest {
    pub topics: Vec<String>,
    pub timeout: i32
}

impl ProtocolSerializable for DeleteTopicsRequest {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let topics_bytes =
            ProtocolArray::of(self.topics.clone()).into_protocol_bytes();

        topics_bytes.and_then(|mut t| {
            I32(self.timeout).into_protocol_bytes().map(|ref mut a| {
                t.append(a);
                t
            })
        })
    }
}