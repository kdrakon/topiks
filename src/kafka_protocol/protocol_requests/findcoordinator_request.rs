use kafka_protocol::api_verification::KafkaApiVersioned;
use kafka_protocol::protocol_primitives::*;
use kafka_protocol::protocol_serializable::*;

#[derive(Clone)]
pub struct FindCoordinatorRequest {
    pub coordinator_key: String,
    pub coordinator_type: i8,
}

pub enum CoordinatorType {
    Group = 0,
    Transaction = 1,
}

impl KafkaApiVersioned for FindCoordinatorRequest {
    fn api_key() -> i16 { 10 }
    fn version() -> i16 { 1 }
}

impl ProtocolSerializable for FindCoordinatorRequest {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        self.coordinator_key.clone().into_protocol_bytes().and_then(|mut coordinator_key| {
            ProtocolPrimitives::I8(self.coordinator_type).into_protocol_bytes().map(|ref mut coordinator_type| {
                coordinator_key.append(coordinator_type);
                coordinator_key
            })
        })
    }
}