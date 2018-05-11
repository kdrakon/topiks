use app_config::AppConfig;
use event_bus::Event::*;
use event_bus::Message::*;
use event_bus::MoveSelection::*;
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
use ui;

pub struct BootstrapServer(pub String);
pub enum MoveSelection {
    Up, Down
}

pub enum Message {
    GetTopics(BootstrapServer),
    SelectTopic(MoveSelection),
    DeleteTopic(BootstrapServer, String),
}

pub enum Event {
    Error(String),
    ListTopics(Response<MetadataResponse>),
    TopicSelected(fn(&State) -> usize)
}

pub struct State {
    pub metadata: Option<MetadataResponse>,
    pub selected_index: usize,
}

impl State {
    fn new() -> State {
        State { metadata: None, selected_index: 0 }
    }
    fn with_metadata(&self, metadata: MetadataResponse) -> State {
        State { metadata: Some(metadata.clone()), selected_index: self.selected_index }
    }
    fn with_selected_index(&self, selected_index: usize) -> State {
        State { metadata: self.metadata.clone(), selected_index }
    }
}

pub fn start() -> Sender<Message> {
    let (sender, receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    thread::spawn(move || {
        let mut state = State::new();
        for message in receiver {
            if let Some(updated_state) = to_event(message).and_then(|event| update_state(event, &state)) {
                state = updated_state;
            }
            ui::update_with_state(&state);
        }
    });

    sender
}

fn to_event(message: Message) -> Option<Event> {
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
        },

        SelectTopic(direction) => {
            match direction {
                Up => Some(TopicSelected(|state: &State|{
                    if state.selected_index > 0 { state.selected_index - 1 } else { state.selected_index }
                })),
                Down => Some(TopicSelected(|state: &State|{
                    match state.metadata {
                        Some(ref metadata) => if state.selected_index < (metadata.topic_metadata.len() - 1) { state.selected_index + 1 } else { state.selected_index },
                        None => 0
                    }
                }))
            }
        },

        DeleteTopic(bootstrap, topic) => {
            None
        }
    }
}

fn update_state(event: Event, current_state: &State) -> Option<State> {
    match event {
        Error(_) => None,
        ListTopics(response) => Some(current_state.with_metadata(response.response_message)),
        TopicSelected(change_fn) => Some(current_state.with_selected_index(change_fn(current_state)))
    }
}

