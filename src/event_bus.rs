use app_config::AppConfig;
use event_bus::Event::*;
use event_bus::Message::*;
use event_bus::MoveSelection::*;
use kafka_protocol::protocol_request::Request;
use kafka_protocol::protocol_requests::deletetopics_request::DeleteTopicsRequest;
use kafka_protocol::protocol_requests::metadata_request::MetadataRequest;
use kafka_protocol::protocol_response::Response;
use kafka_protocol::protocol_responses::deletetopics_response::DeleteTopicsResponse;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use state::*;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use tcp_stream_util;
use tcp_stream_util::TcpRequestError;
use ui;

pub struct BootstrapServer(pub String);

pub enum MoveSelection { Up, Down }

pub enum Message {
    GetTopics(BootstrapServer),
    SelectTopic(MoveSelection),
    DeleteTopic(BootstrapServer),
    ToggleTopicInfo(),
}

enum Event {
    Error(String),
    ListTopics(Response<MetadataResponse>),
    TopicSelected(fn(&State) -> usize),
    TopicDeleted(Box<Fn(&State) -> (usize, bool)>),
    InfoToggled(fn(&State) -> bool),
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
                    Some(Error(e.error))
                }
            }
        }

        SelectTopic(direction) => {
            match direction {
                Up => Some(TopicSelected(|state: &State| {
                    if state.selected_index > 0 { state.selected_index - 1 } else { state.selected_index }
                })),
                Down => Some(TopicSelected(|state: &State| {
                    match state.metadata {
                        Some(ref metadata) => if state.selected_index < (metadata.topic_metadata.len() - 1) { state.selected_index + 1 } else { state.selected_index },
                        None => 0
                    }
                }))
            }
        }

        DeleteTopic(BootstrapServer(bootstrap)) => {
            Some(TopicDeleted(Box::from(move |state: &State| {
                match state.metadata {
                    Some(ref metadata) => {
                        match metadata.topic_metadata.get(state.selected_index) {
                            Some(delete_topic_metadata) => {
                                let result: Result<Response<DeleteTopicsResponse>, TcpRequestError> = tcp_stream_util::request(
                                    bootstrap.clone(),
                                    Request::of(DeleteTopicsRequest { topics: vec![delete_topic_metadata.topic.clone()], timeout: 30_000 }, 20, 1),
                                );

                                match result {
                                    Ok(response) => {
                                        (state.selected_index, response.response_message.topic_error_codes.iter().all(|err| err.error_code == 0))
                                    }
                                    Err(err) => {
                                        eprintln!("{}", err.error);
                                        (state.selected_index, false)
                                    }
                                }
                            }
                            None => (state.selected_index, false)
                        }
                    }
                    None => (state.selected_index, false)
                }
            })))
        }

        ToggleTopicInfo() => Some(InfoToggled(move |state| {
            !state.show_selected_topic_info
        }))
    }
}

fn update_state(event: Event, current_state: &State) -> Option<State> {
    match event {
        Error(_) => None,
        ListTopics(response) => Some(current_state.with_metadata(response.response_message).with_marked_deleted(vec![])),
        TopicSelected(select_fn) => Some(current_state.with_selected_index(select_fn(current_state))),
        TopicDeleted(boxed_delete_fn) => {
            let (selected, deleted) = boxed_delete_fn(current_state);
            match deleted {
                false => None,
                true => {
                    let mut deleted = current_state.marked_deleted.clone();
                    deleted.push(selected);
                    Some(current_state.with_marked_deleted(deleted))
                }
            }
        }
        InfoToggled(toggle_fn) => Some(current_state.with_show_selected_topic_info(toggle_fn(current_state)))
    }
}

