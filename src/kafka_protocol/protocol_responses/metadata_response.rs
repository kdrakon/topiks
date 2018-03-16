extern crate byteorder;

use kafka_protocol::protocol_response::*;
use kafka_protocol::protocol_serializable::*;

pub struct MetadataResponse {
    throttle_time_ms: i32,
    brokers: Vec<BrokerMetadata>,
    cluster_id: Option<String>,
    controller_id: i32,
    topic_metadata: Vec<TopicMetadata>,
}

pub struct BrokerMetadata {
    node_id: i32,
    host: String,
    port: i32,
    rack: Option<String>,
}

pub struct TopicMetadata {
    error_code: i16,
    topic: String,
    is_internal: bool,
    partition_metadata: Vec<PartitionMetadata>,
}

pub struct PartitionMetadata {
    error_code: i16,
    partition: i32,
    leader: i32,
    replicas: Vec<i32>,
    isr: Vec<i32>,
    offline_replicas: Vec<i32>,
}

impl ProtocolDeserializable<Response<MetadataResponse>> for Vec<u8> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<Response<MetadataResponse>> {
        let fields: ProtocolDeserializeResult<(ResponseHeader, MetadataResponse)> =
            self[0..4].to_vec().into_protocol_type().and_then(|header| {
                self[4..].to_vec().into_protocol_type().map(|response_message| {
                    (header, response_message)
                })
            });

        fields.map(|(header, response_message)| {
            Response {
                header,
                response_message,
            }
        })
    }
}

impl ProtocolDeserializable<MetadataResponse> for Vec<u8> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<MetadataResponse> {
        let result =
            de_i32(self[0..4].to_vec()).and_then(|throttle_time_ms| {
                de_array(self[4..].to_vec(), deserialize_broker_metadata).map(|(brokers, remaining_bytes): (Vec<BrokerMetadata>, Vec<u8>)| {
                    (throttle_time_ms, brokers, remaining_bytes)
                })
            });

        let result =
            result.and_then(|(throttle_time_ms, brokers, remaining_bytes)| {
                de_string(remaining_bytes).map(|(cluster_id, remaining_bytes)| {
                    (throttle_time_ms, brokers, cluster_id, remaining_bytes)
                })
            });

        let result =
            result.and_then(|(throttle_time_ms, brokers, cluster_id, remaining_bytes)| {
                de_i32(remaining_bytes[0..4].to_vec()).map(|controller_id| {
                    (throttle_time_ms, brokers, cluster_id, controller_id, remaining_bytes[4..].to_vec())
                })
            });

        let result =
            result.and_then(|(throttle_time_ms, brokers, cluster_id, controller_id, remaining_bytes)| {
                de_array(remaining_bytes, deserialize_topic_metadata).map(|(topic_metadata, remaining_bytes)| {
                    (throttle_time_ms, brokers, cluster_id, controller_id, topic_metadata, remaining_bytes)
                })
            });

        result.map(|(throttle_time_ms, brokers, cluster_id, controller_id, topic_metadata, remaining_bytes)| {
            if !remaining_bytes.is_empty() { panic!("Deserialize of MetadataResponse did not cover remaining {} bytes", remaining_bytes.len()); }
            MetadataResponse { throttle_time_ms, brokers, cluster_id, controller_id, topic_metadata }
        })
    }
}

fn deserialize_broker_metadata(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicType<BrokerMetadata>> {
    let result =
        de_i32(bytes[0..4].to_vec()).and_then(|node_id| {
            de_string(bytes[4..].to_vec()).map(|(host, remaining_bytes)| {
                let host = host.expect("Expected host string");
                (node_id, host, remaining_bytes)
            })
        });

    let result =
        result.and_then(|(node_id, host, remaining_bytes)| {
            de_i32(remaining_bytes[0..4].to_vec()).and_then(|port| {
                de_string(remaining_bytes[4..].to_vec()).map(|(rack, remaining_bytes)| {
                    (node_id, host, port, rack, remaining_bytes)
                })
            })
        });

    result.map(|(node_id, host, port, rack, remaining_bytes)| {
        (BrokerMetadata { node_id, host, port, rack }, remaining_bytes)
    })
}

fn deserialize_topic_metadata(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicType<TopicMetadata>> {
    de_i16(bytes[0..2].to_vec()).and_then(|error_code| {
        de_string(bytes[2..].to_vec()).map(|(topic, remaining_bytes)| {
            let topic = topic.expect("Unexpected empty topic name");
            let is_internal = if remaining_bytes[0] == 1 { true } else { false };
            (topic, is_internal, remaining_bytes[1..].to_vec())
        }).and_then(|(topic, is_internal, remaining_bytes)| {
            de_array(remaining_bytes, deserialize_partition_metadata).map(|(partition_metadata, remaining_bytes)| {
                (TopicMetadata { error_code, topic, is_internal, partition_metadata }, remaining_bytes)
            })
        })
    })
}

fn deserialize_partition_metadata(bytes: Vec<u8>) -> ProtocolDeserializeResult<DynamicType<PartitionMetadata>> {
    de_i16(bytes[0..2].to_vec()).and_then(|error_code| {
        de_i32(bytes[2..6].to_vec()).and_then((|partition| {
            de_i32(bytes[6..10].to_vec()).and_then(|leader| {
                de_array(bytes[10..].to_vec(), |bytes| {
                    de_i32(bytes[0..4].to_vec()).map(|replicas| { (replicas, bytes[4..].to_vec()) })
                }).and_then(|(replicas, remaining_bytes)| {
                    de_array(remaining_bytes, |bytes| {
                        de_i32(bytes[0..4].to_vec()).map(|isr| { (isr, bytes[4..].to_vec()) })
                    }).and_then(|(isr, remaining_bytes)| {
                        de_array(remaining_bytes, |bytes| {
                            de_i32(bytes[0..4].to_vec()).map(|offline_replicas| { (offline_replicas, bytes[4..].to_vec()) })
                        }).map(|(offline_replicas, remaining_bytes)| {
                            (
                                PartitionMetadata { error_code, partition, leader, replicas, isr, offline_replicas },
                                remaining_bytes
                            )
                        })
                    })
                })
            })
        }))
    })
}
