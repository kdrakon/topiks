use std::collections::HashMap;

use api_client::ApiClientTrait;
use api_client::ApiRequestError;
use event_bus;
use event_bus::*;
use kafka_protocol::protocol_request::Request;
use kafka_protocol::protocol_response::Response;
use kafka_protocol::protocol_serializable::ProtocolDeserializable;
use kafka_protocol::protocol_serializable::ProtocolSerializable;
use state::State;
use util::io::IO;
use BootstrapServer;

struct FakeApiClient(HashMap<i16, Vec<u8>>); // ApiKey => Byte Response

impl ApiClientTrait for FakeApiClient {
    fn request<T, U>(&self, bootstrap_server: &BootstrapServer, request: Request<T>) -> Result<Response<U>, ApiRequestError>
    where
        T: ProtocolSerializable,
        Vec<u8>: ProtocolDeserializable<Response<U>>,
    {
        dbg!(bootstrap_server);
        let response = self.0.get(&request.header.api_key).expect("ApiKey response not defined");
        response.clone().into_protocol_type().map_err(|e| ApiRequestError::of(e.error))
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

    match event_bus::to_event(Message::GetMetadata(BootstrapServer::of(String::from("fake"), 9092, false), None), test_api_client_provider) {
        Event::MetadataRetrieved(statefn) => match statefn(&state) {
            Ok(_metadata_payload) => (),
            Err(_e) => panic!(),
        },
        _ => panic!("Expected MetadataRetrieved"),
    }
}
