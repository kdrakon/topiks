use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::{Cursor, Read, Write};
use std::net::*;

use byteorder::{BigEndian, ReadBytesExt};
use native_tls::TlsConnector;

use kafka_protocol::protocol_request::*;
use kafka_protocol::protocol_response::*;
use kafka_protocol::protocol_serializable::*;
use util::io::IO;
use BootstrapServer;

#[derive(Debug)]
pub struct ApiRequestError {
    pub error: String,
}

impl Display for ApiRequestError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "API Request Error: {}", self.error)
    }
}

impl ApiRequestError {
    pub fn of(error: String) -> ApiRequestError {
        ApiRequestError { error }
    }
    pub fn from(error: &str) -> ApiRequestError {
        ApiRequestError::of(String::from(error))
    }
}

pub trait ApiClientTrait {
    fn request<T, U>(&self, bootstrap_server: &BootstrapServer, request: Request<T>) -> Result<Response<U>, ApiRequestError>
    where
        T: ProtocolSerializable,
        Vec<u8>: ProtocolDeserializable<Response<U>>;
}

pub type ApiClientProvider<T> = Box<Fn() -> IO<T, ApiRequestError>>;

#[derive(Clone)]
pub struct ApiClient {}

impl ApiClient {
    pub fn new() -> ApiClient {
        ApiClient {}
    }
}

impl ApiClientTrait for ApiClient {
    fn request<T, U>(&self, bootstrap_server: &BootstrapServer, request: Request<T>) -> Result<Response<U>, ApiRequestError>
    where
        T: ProtocolSerializable,
        Vec<u8>: ProtocolDeserializable<Response<U>>,
    {
        let _api_key = request.header.api_key;
        let _api_key_version = request.header.api_version;
        let request_bytes = request.into_protocol_bytes().map_err(|err| ApiRequestError::of(format!("Could not serialize request. {}", err)));

        fn tcp_stream(bootstrap_server: &BootstrapServer) -> Result<TcpStream, ApiRequestError> {
            TcpStream::connect(bootstrap_server.as_socket_addr()).map_err(|err| ApiRequestError::of(err.to_string()))
        }

        fn write_bytes<S: Write>(stream: &mut S, bytes: Vec<u8>) -> Result<usize, ApiRequestError> {
            stream.write(bytes.as_slice()).map_err(|err| ApiRequestError::of(err.to_string()))
        }

        fn read_bytes<S: Read>(stream: &mut S) -> Result<Vec<u8>, ApiRequestError> {
            let mut result_size_buf: [u8; 4] = [0; 4];
            stream
                .read(&mut result_size_buf)
                .and_then(|_| Cursor::new(result_size_buf.to_vec()).read_i32::<BigEndian>())
                .and_then(|result_size| {
                    let mut message_buf: Vec<u8> = vec![0; result_size as usize];
                    stream.read_exact(&mut message_buf).map(|_| message_buf)
                })
                .map_err(|err| ApiRequestError::of(err.to_string()))
        }

        let response = request_bytes.and_then(|bytes| match bootstrap_server.use_tls {
            false => tcp_stream(bootstrap_server).and_then(|ref mut stream| write_bytes(stream, bytes).and_then(|_| read_bytes(stream))),
            true => tcp_stream(bootstrap_server).and_then(|stream| {
                TlsConnector::new()
                    .map_err(|err| ApiRequestError::of(err.to_string()))
                    .and_then(|tls_connector| {
                        tls_connector
                            .connect(bootstrap_server.domain.as_str(), stream)
                            .map_err(|err| ApiRequestError::of(format!("TLS handshake error. {}", err)))
                    })
                    .and_then(|ref mut stream| write_bytes(stream, bytes).and_then(|_| read_bytes(stream)))
            }),
        });

        // DEBUG response
        response.iter().for_each(|bytes| {
            use util::utils;
            println!("bytes for API key {}:v{}: {:?}", _api_key, _api_key_version, utils::to_hex_array(&bytes));
        });

        response.and_then(|bytes| bytes.into_protocol_type().map_err(|e| ApiRequestError::of(e.error)))
    }
}
