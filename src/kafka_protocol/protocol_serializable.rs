/// If implemented, a struct/enum can be sent on the wire to a
/// Kafka broker.
///
pub trait ProtocolSerializable {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult;
}

pub type ProtocolSerializeResult = Result<Vec<u8>, ProtocolSerializeError>;

#[derive(Debug)]
pub struct ProtocolSerializeError(pub String);