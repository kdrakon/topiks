use api_client::ApiClientTrait;
use api_client::TcpRequestError;
use kafka_protocol::api_verification::ApiVerificationFailure::ApiNotSupported;
use kafka_protocol::api_verification::ApiVerificationFailure::ApiVersionNotSupported;
use kafka_protocol::api_verification::ApiVerificationFailure::NoVerification;
use kafka_protocol::protocol_request::Request;
use kafka_protocol::protocol_requests::alterconfigs_request::AlterConfigsRequest;
use kafka_protocol::protocol_requests::deletetopics_request::DeleteTopicsRequest;
use kafka_protocol::protocol_requests::describeconfigs_request::DescribeConfigsRequest;
use kafka_protocol::protocol_requests::findcoordinator_request::FindCoordinatorRequest;
use kafka_protocol::protocol_requests::listoffsets_request::ListOffsetsRequest;
use kafka_protocol::protocol_requests::metadata_request::MetadataRequest;
use kafka_protocol::protocol_requests::offsetfetch_request::OffsetFetchRequest;
use kafka_protocol::protocol_response::Response;
use kafka_protocol::protocol_serializable::*;
use BootstrapServer;

#[derive(Debug)]
pub enum ApiVerificationFailure {
    NoVerification(String),
    ApiNotSupported(i16),
    ApiVersionNotSupported(i16, i16),
}

#[derive(Clone)]
pub struct ApiVersionsRequest {}

impl KafkaApiVersioned for ApiVersionsRequest {
    fn api_key() -> i16 {
        18
    }
    fn version() -> i16 {
        0
    }
}

impl ProtocolSerializable for ApiVersionsRequest {
    fn into_protocol_bytes(self) -> ProtocolSerializeResult {
        Ok(vec![])
    }
}

#[derive(Debug)]
pub struct ApiVersionResponse {
    pub error_code: i16,
    pub api_versions: Vec<ApiVersion>,
}

#[derive(Debug)]
pub struct ApiVersion {
    pub api_key: i16,
    pub min_version: i16,
    pub max_version: i16,
}

impl ProtocolDeserializable<ApiVersionResponse> for Vec<u8> {
    fn into_protocol_type(self) -> ProtocolDeserializeResult<ApiVersionResponse> {
        fn deserialize_api_version(
            bytes: Vec<u8>,
        ) -> ProtocolDeserializeResult<DynamicSize<ApiVersion>> {
            de_i16(bytes[0..=1].to_vec()).and_then(|api_key| {
                de_i16(bytes[2..=3].to_vec()).and_then(|min_version| {
                    de_i16(bytes[4..=5].to_vec()).map(|max_version| {
                        (
                            ApiVersion {
                                api_key,
                                min_version,
                                max_version,
                            },
                            bytes[6..].to_vec(),
                        )
                    })
                })
            })
        }

        de_i16(self[0..=1].to_vec()).and_then(|error_code| {
            de_array(self[2..].to_vec(), deserialize_api_version).map(|(api_versions, _bytes)| {
                ApiVersionResponse {
                    error_code,
                    api_versions,
                }
            })
        })
    }
}

pub struct ApiVersionQuery(pub i16, pub i16); // api -> version

pub fn apply<T: ApiClientTrait + 'static>(
    api_client: T,
    bootstrap_server: &BootstrapServer,
    queries: &Vec<ApiVersionQuery>,
) -> Result<(), Vec<ApiVerificationFailure>> {
    let result: Result<Response<ApiVersionResponse>, TcpRequestError> =
        api_client.request(bootstrap_server, Request::of(ApiVersionsRequest {}));

    let verification = result
        .map(|response| response.response_message.api_versions)
        .map(|api_versions| {
            let api_errors: Vec<ApiVerificationFailure> = vec![];
            queries.iter().fold(api_errors, |mut api_errors, query| {
                let verification = match api_versions
                    .iter()
                    .find(|version| version.api_key == query.0)
                {
                    Some(version) => {
                        let verified = version.api_key == query.0
                            && query.1 >= version.min_version
                            && query.1 <= version.max_version;
                        if !verified {
                            Some(ApiVersionNotSupported(query.0, query.1))
                        } else {
                            None
                        }
                    }
                    None => Some(ApiNotSupported(query.0)),
                };

                match verification {
                    Some(error) => {
                        api_errors.push(error);
                        api_errors
                    }
                    None => api_errors,
                }
            })
        });

    match verification {
        Ok(failures) => match failures.as_slice() {
            [] => Ok(()),
            _ => Err(failures),
        },
        Err(err) => Err(vec![NoVerification(err.error)]),
    }
}

pub trait KafkaApiVersioned {
    fn api_key() -> i16;
    fn version() -> i16;
}

pub fn apis_in_use() -> Vec<ApiVersionQuery> {
    vec![
        ApiVersionQuery(
            AlterConfigsRequest::api_key(),
            AlterConfigsRequest::version(),
        ),
        ApiVersionQuery(
            DeleteTopicsRequest::api_key(),
            DeleteTopicsRequest::version(),
        ),
        ApiVersionQuery(
            DescribeConfigsRequest::api_key(),
            DescribeConfigsRequest::version(),
        ),
        ApiVersionQuery(
            FindCoordinatorRequest::api_key(),
            FindCoordinatorRequest::version(),
        ),
        ApiVersionQuery(ListOffsetsRequest::api_key(), ListOffsetsRequest::version()),
        ApiVersionQuery(MetadataRequest::api_key(), MetadataRequest::version()),
        ApiVersionQuery(OffsetFetchRequest::api_key(), OffsetFetchRequest::version()),
    ]
}
