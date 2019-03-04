use kafka_protocol::api_verification::KafkaApiVersioned;
use kafka_protocol::protocol_primitives::ProtocolPrimitives::*;
use kafka_protocol::protocol_serializable::ProtocolSerializeResult;
use kafka_protocol::protocol_serializable::*;

/// Top-level request which can be sent to a Kafka broker.
///
#[derive(Clone)]
pub struct Request<T: ProtocolSerializable> {
    pub header: RequestHeader,
    pub request_message: T,
}

impl<A: ProtocolSerializable + KafkaApiVersioned> Request<A> {
    pub fn of(request_message: A) -> Request<A> {
        Request {
            header: RequestHeader { api_key: A::api_key(), api_version: A::version(), correlation_id: 42, client_id: String::from("topiks") },
            request_message,
        }
    }
}

impl<T> ProtocolSerializable for Request<T>
where
    T: ProtocolSerializable,
{
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        let header = self.header;
        let request_message = self.request_message;
        let header_and_request = header.into_protocol_bytes().and_then(|mut h| {
            request_message.into_protocol_bytes().map(|ref mut rm| {
                h.append(rm);
                h
            })
        });

        header_and_request.and_then(|ref mut hr| {
            I32(hr.len() as i32).into_protocol_bytes().map(|mut message_size| {
                message_size.append(hr);
                message_size
            })
        })
    }
}

/// Header information for a Request
///
#[derive(Clone)]
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
