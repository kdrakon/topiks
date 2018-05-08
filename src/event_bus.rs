use app_config::AppConfig;
use event_bus::Message::*;
use event_bus::Event::*;
use kafka_protocol::protocol_request::Request;
use kafka_protocol::protocol_requests::metadata_request::MetadataRequest;
use kafka_protocol::protocol_response::Response;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use tcp_stream_util;
use tcp_stream_util::TcpRequestError;

pub struct BootstrapServer(pub String);

pub enum Message {
    GetTopics(BootstrapServer),
    DeleteTopic(BootstrapServer, String),
}

pub enum Event {
    Error(String),
    ListTopics(Response<MetadataResponse>),
}

pub fn start() -> Sender<Message> {
    let (sender, receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    thread::spawn(move || {
        for message in receiver {
            from_message(message);
        }
    });

    sender
}

fn from_message(message: Message) -> Option<Event> {
    match message {
        GetTopics(BootstrapServer(bootstrap)) => {
            let result: Result<Response<MetadataResponse>, TcpRequestError> =
                tcp_stream_util::request(
                    bootstrap,
                    Request::of(MetadataRequest { topics: None, allow_auto_topic_creation: false }, 3, 5),
                );
            match result {
                Ok(response) => Some(ListTopics(response)),
                Err(e) => {
                    eprintln!("{}", e.error);
                    None
                }
            }
        }

        DeleteTopic(bootstrap, topic) => {
            None
        }
    }
}

