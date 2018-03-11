use kafka_protocol::protocol_primitives::*;
use kafka_protocol::protocol_primitives::ProtocolPrimitives::*;
use kafka_protocol::protocol_serializable::*;
use kafka_protocol::protocol_serializable::ProtocolSerializeResult;
use self::RequestMessage::*;

/// Top-level request which can be sent to a Kafka broker.
///
pub struct Request {
    pub header: RequestHeader,
    pub request_message: RequestMessage,
}

impl ProtocolSerializable for Request {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let header = self.header;
        let request_message = self.request_message;
        let header_and_request =
            header.into_protocol_bytes().and_then(|mut h|
                request_message.into_protocol_bytes().map(|ref mut rm| {
                    h.append(rm);
                    h
                })
            );

        header_and_request.and_then(|ref mut hr|
            I32(hr.len() as i32).into_protocol_bytes().map(|mut message_size| {
                message_size.append(hr);
                message_size
            }))
    }
}

/// Header information for a Request
///
pub struct RequestHeader {
    pub api_key: i16,
    pub api_version: i16,
    pub correlation_id: i32,
    pub client_id: String,
}

impl ProtocolSerializable for RequestHeader {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        I16(self.api_key).into_protocol_bytes().and_then(|mut api_key| {
            I16(self.api_version).into_protocol_bytes().and_then(|ref mut api_version| {
                I32(self.correlation_id).into_protocol_bytes().and_then(|ref mut correlation_id| {
                    self.client_id.into_protocol_bytes().and_then(|ref mut client_id| {
                        api_key.append(api_version);
                        api_key.append(correlation_id);
                        api_key.append(client_id);
                        Ok(api_key.clone())
                    })
                })
            })
        })
    }
}

/// Enum of requests that can be sent to the Kafka API
///
pub enum RequestMessage {
    MetadataRequest { topics: Option<ProtocolArray<String>>, allow_auto_topic_creation: bool }
}

impl ProtocolSerializable for RequestMessage {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        match self {
            MetadataRequest { topics, allow_auto_topic_creation } => RequestMessage::serialize_metadata_request(topics, allow_auto_topic_creation)
        }
    }
}

impl RequestMessage {
    fn serialize_metadata_request(topics: Option<ProtocolArray<String>>, allow_auto_topic_creation: bool) -> ProtocolSerializeResult {
        let topic_bytes =
            match topics {
                Some(topics) => topics.into_protocol_bytes(),
                None => I32(-1).into_protocol_bytes()
            };

        topic_bytes.and_then(|mut t| {
            Boolean(allow_auto_topic_creation).into_protocol_bytes().map(|ref mut a| {
                t.append(a);
                t
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_request() {
        let topics: Option<ProtocolArray<String>> = Some(ProtocolArray::of(vec![]));
        let metadata_request = RequestMessage::MetadataRequest { topics, allow_auto_topic_creation: false };

        let request =
            Request {
                header: RequestHeader {
                    api_key: 3,
                    api_version: 5,
                    correlation_id: 42,
                    client_id: String::from("sean"),
                },
                request_message: metadata_request,
            };

        let bytes = request.into_protocol_bytes().unwrap();
        assert_eq!(vec![0, 0, 0, 19, 0, 3, 0, 5, 0, 0, 0, 42, 0, 4, 115, 101, 97, 110, 0, 0, 0, 0, 0], bytes);
        println!("{:?}", bytes);
    }

    #[test]
    fn verify_metadata_request() {
        let topics = Some(ProtocolArray::of(vec! {
            String::from("my_kafka_topic_1"),
            String::from("my_kafka_topic_2")
        }));
        let metadata_request = RequestMessage::MetadataRequest { topics, allow_auto_topic_creation: true };
        let expected: Vec<u8> = vec![0, 0, 0, 2,
                                     0, 16, 0x6D, 0x79, 0x5F, 0x6B, 0x61, 0x66, 0x6B, 0x61, 0x5F, 0x74, 0x6F, 0x70, 0x69, 0x63, 0x5F, 0x31,
                                     0, 16, 0x6D, 0x79, 0x5F, 0x6B, 0x61, 0x66, 0x6B, 0x61, 0x5F, 0x74, 0x6F, 0x70, 0x69, 0x63, 0x5F, 0x32,
                                     1];
        assert_eq!(expected, metadata_request.into_protocol_bytes().unwrap());
    }
}