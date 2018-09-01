use kafka_protocol::protocol_request::Request;
use kafka_protocol::protocol_response::Response;
use kafka_protocol::protocol_serializable::*;
use util;
use util::tcp_stream_util::TcpRequestError;
use util::utils;

pub enum ApiVerification {
    ApiNotSupported(String),
    ApiVersionNotSupported(String, i32),
}

/// Version 2
///
#[derive(Clone)]
pub struct ApiVersionsRequest {}

impl ProtocolSerializable for ApiVersionsRequest {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        Ok(vec![])
    }
}

/// Version 2
///
pub struct ApiVersionResponse {
    pub error_code: i16,
    pub api_versions: Vec<ApiVersion>,
    pub throttle_time_ms: i32,
}

pub struct ApiVersion {
    pub api_key: i16,
    pub min_version: i16,
    pub max_version: i16,
}

impl ProtocolDeserializable<ApiVersionResponse> for Vec<u8> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<ApiVersionResponse> {
        println!("bytes: {:?}", utils::to_hex_array(&self));
        Ok(ApiVersionResponse {
            error_code: 1,
            api_versions: vec![],
            throttle_time_ms: 32,
        })
    }
}

pub fn apply(bootstrap_server: &str) -> Result<(), ApiVerification> {
    let result: Result<Response<ApiVersionResponse>, TcpRequestError> =
        util::tcp_stream_util::request(
            bootstrap_server.clone(),
            Request::of(ApiVersionsRequest {}, 18, 2),
        );

    Ok(())
}