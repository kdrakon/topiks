use std::collections::HashMap;
use std::net::ToSocketAddrs;

use api_client::ApiClientTrait;
use api_client::TcpRequestError;
use event_bus;
use event_bus::*;
use kafka_protocol::protocol_request::Request;
use kafka_protocol::protocol_response::Response;
use kafka_protocol::protocol_serializable::ProtocolDeserializable;
use kafka_protocol::protocol_serializable::ProtocolSerializable;
use state::State;
use util::io::IO;

struct FakeApiClient(HashMap<i16, Vec<u8>>); // ApiKey => Byte Response

impl ApiClientTrait for FakeApiClient {
    fn request<A, T, U>(&self, _address: A, request: Request<T>) -> Result<Response<U>, TcpRequestError>
    where
        A: ToSocketAddrs,
        T: ProtocolSerializable,
        Vec<u8>: ProtocolDeserializable<Response<U>>,
    {
        let response = self.0.get(&request.header.api_key).expect("ApiKey response not defined");
        response.clone().into_protocol_type().map_err(|e| TcpRequestError::of(e.error))
    }
}

#[test]
#[ignore] // imcomplete
fn get_metadata() {
    let state = State::new();

    let test_api_client_provider = Box::new(|| {
        IO::new(Box::new(|| {
            Ok(FakeApiClient({
                let mut responses: HashMap<i16, Vec<u8>> = HashMap::new();
                responses.insert(3, vec![0, 2]);
                responses
            }))
        }))
    });

    match event_bus::to_event(Message::GetMetadata(BootstrapServer(String::from("fake")), None), test_api_client_provider) {
        Event::MetadataRetrieved(statefn) => match statefn(&state) {
            Ok(_metadata_payload) => (),
            Err(_e) => panic!(),
        },
        _ => panic!("Expected MetadataRetrieved"),
    }
}
