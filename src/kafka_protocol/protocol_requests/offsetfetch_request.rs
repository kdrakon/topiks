use kafka_protocol::protocol_primitives::*;
use kafka_protocol::protocol_primitives::ProtocolPrimitives::I32;
use kafka_protocol::protocol_serializable::ProtocolSerializable;
use kafka_protocol::protocol_serializable::ProtocolSerializeResult;

/// Version 3
pub struct OffsetFetchRequest {
    pub group_id: String,
    pub topics: Topic,
}

pub struct Topic {
    pub topic: String,
    pub partitions: Vec<i32>,
}

impl ProtocolSerializable for OffsetFetchRequest {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let group_id = self.group_id;
        let topics = self.topics;
        group_id.into_protocol_bytes().and_then(|mut group_id| {
            topics.into_protocol_bytes().map(|ref mut topics| {
                group_id.append(topics);
                group_id
            })
        })
    }
}

impl ProtocolSerializable for Topic {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let topic = self.topic;
        let partitions = self.partitions;
        topic.into_protocol_bytes().and_then(|mut topic| {
            partitions.into_iter()
                .map(|p| I32(p)).collect::<Vec<ProtocolPrimitives>>()
                .into_protocol_bytes()
                .map(|ref mut partitions| {
                    topic.append(partitions);
                    topic
                })
        })
    }
}