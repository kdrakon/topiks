use app_config::AppConfig;
use event_bus::Event::*;
use event_bus::Message::*;
use event_bus::MoveSelection::*;
use kafka_protocol::protocol_request::Request;
use kafka_protocol::protocol_requests::deletetopics_request::DeleteTopicsRequest;
use kafka_protocol::protocol_requests::describeconfigs_request::DescribeConfigsRequest;
use kafka_protocol::protocol_requests::describeconfigs_request::Resource;
use kafka_protocol::protocol_requests::describeconfigs_request::ResourceTypes;
use kafka_protocol::protocol_requests::metadata_request::MetadataRequest;
use kafka_protocol::protocol_response::Response;
use kafka_protocol::protocol_responses::deletetopics_response::DeleteTopicsResponse;
use kafka_protocol::protocol_responses::describeconfigs_response::DescribeConfigsResponse;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use state::*;
use std::cell::RefCell;
use std::cell::RefMut;
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
    ToggleTopicInfo(BootstrapServer)
}

enum Event {
    Error(String),
    ListTopics(Response<MetadataResponse>),
    TopicSelected(fn(&State) -> usize),
    TopicDeleted(Box<Fn(&State) -> (usize, bool)>),
    InfoToggled(Box<Fn(&State) -> Option<TopicInfoState>>)
}

pub fn start() -> Sender<Message> {
    let (sender, receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    thread::spawn(move || {
        let state = RefCell::new(State::new()); // RefCell for interior mutability ('unsafe' code)
        for message in receiver {
            if let Some(updated_state) = to_event(message).and_then(|event| update_state(event, state.borrow_mut())) {
                state.swap(&RefCell::new(updated_state));
            }
            ui::update_with_state(&state.borrow());
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
                                let result: Result<Response<DeleteTopicsResponse>, TcpRequestError> =
                                    tcp_stream_util::request(
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

        ToggleTopicInfo(BootstrapServer(bootstrap)) =>
            Some(InfoToggled(Box::from(move |state: &State| {
                match state.topic_info_state {
                    Some(_) => None,
                    None => {
                        let resource = Resource {
                            resource_type: ResourceTypes::Topic as i8,
                            resource_name: state.selected_topic_name(),
                            config_names: None,
                        };
                        let result: Result<Response<DescribeConfigsResponse>, TcpRequestError> =
                            tcp_stream_util::request(
                                bootstrap.clone(),
                                Request::of(DescribeConfigsRequest { resources: vec![resource], include_synonyms: false }, 32, 1),
                            );
                        match result {
                            Ok(response) => {
                                Some(TopicInfoState { config_info: response.response_message.resources })
                            },
                            Err(err) => {
                                eprintln!("{}", err.error);
                                None
                            }
                        }
                    }
                }
            })))
    }
}

fn update_state(event: Event, mut current_state: RefMut<State>) -> Option<State> {
    match event {
        Error(_) => None,
        ListTopics(response) => {
            current_state.metadata = Some(response.response_message);
            current_state.marked_deleted = vec![];
            Some(current_state.clone())
        }
        TopicSelected(select_fn) => {
            current_state.selected_index = select_fn(&current_state);
            Some(current_state.clone())
        }
        TopicDeleted(boxed_delete_fn) => {
            let (selected, deleted) = boxed_delete_fn(&current_state);
            match deleted {
                false => None,
                true => {
                    current_state.marked_deleted.push(selected);
                    Some(current_state.clone())
                }
            }
        }
        InfoToggled(toggle_fn) => {
            current_state.topic_info_state = toggle_fn(&current_state);
            Some(current_state.clone())
        }
    }
}

