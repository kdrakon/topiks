use kafka_protocol::api_verification::KafkaApiVersioned;
use kafka_protocol::protocol_primitives::ProtocolPrimitives::*;
use kafka_protocol::protocol_serializable::*;

#[derive(Clone)]
pub struct ListOffsetsRequest {
    pub replica_id: i32,
    pub isolation_level: i8,
    pub topics: Vec<Topic>,
}

#[derive(Clone, Debug)]
pub struct Topic {
    pub topic: String,
    pub partitions: Vec<Partition>,
}

#[derive(Clone, Debug)]
pub struct Partition {
    pub partition: i32,
    pub timestamp: i64,
}

impl KafkaApiVersioned for ListOffsetsRequest {
    fn api_key() -> i16 {
        2
    }
    fn version() -> i16 {
        2
    }
}

impl ProtocolSerializable for ListOffsetsRequest {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let replica_id = self.replica_id;
        let isolation_level = self.isolation_level;
        let topics = self.topics;
        I32(replica_id).into_protocol_bytes().and_then(|mut replica_id| {
            I8(isolation_level).into_protocol_bytes().and_then(|ref mut isolation_level| {
                topics.into_protocol_bytes().map(|ref mut topics| {
                    replica_id.append(isolation_level);
                    replica_id.append(topics);
                    replica_id
                })
            })
        })
    }
}

impl ProtocolSerializable for Topic {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let topic = self.topic;
        let partitions = self.partitions;
        topic.into_protocol_bytes().and_then(|mut topic| {
            partitions.into_protocol_bytes().map(|ref mut partitions| {
                topic.append(partitions);
                topic
            })
        })
    }
}

impl ProtocolSerializable for Partition {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let partition = self.partition;
        let timestamp = self.timestamp;
        I32(partition).into_protocol_bytes().and_then(|mut partition| {
            I64(timestamp).into_protocol_bytes().map(|ref mut timestamp| {
                partition.append(timestamp);
                partition
            })
        })
    }
}
