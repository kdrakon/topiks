use self::protocol_primitives::*;
use self::protocol_primitives::ProtocolPrimitives::*;
use self::protocol_serializable::*;
use self::RequestMessage::*;

mod protocol_primitives;
mod protocol_serializable;

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
    MetadataRequest { topics: ProtocolArray<String>, allow_auto_topic_creation: bool }
}

impl ProtocolSerializable for RequestMessage {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        match self {
            MetadataRequest { topics, allow_auto_topic_creation } => RequestMessage::serialize_metadata_request(topics, allow_auto_topic_creation)
        }
    }
}

impl RequestMessage {
    fn serialize_metadata_request(topics: ProtocolArray<String>, allow_auto_topic_creation: bool) -> ProtocolSerializeResult {
        topics.into_protocol_bytes().and_then(|mut t| {
            Boolean(allow_auto_topic_creation).into_protocol_bytes().map(|ref mut a| {
                t.append(a);
                t
            })
        })
    }
}