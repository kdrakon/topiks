use app_config::AppConfig;
use event_bus::Event::*;
use event_bus::Message::*;
use event_bus::MoveSelection::*;
use event_bus::TopicQuery::*;
use kafka_protocol::protocol_request::Request;
use kafka_protocol::protocol_requests::deletetopics_request::DeleteTopicsRequest;
use kafka_protocol::protocol_requests::describeconfigs_request::DescribeConfigsRequest;
use kafka_protocol::protocol_requests::describeconfigs_request::Resource;
use kafka_protocol::protocol_requests::describeconfigs_request::ResourceTypes;
use kafka_protocol::protocol_requests::metadata_request::MetadataRequest;
use kafka_protocol::protocol_response::Response;
use kafka_protocol::protocol_responses::deletetopics_response::DeleteTopicsResponse;
use kafka_protocol::protocol_responses::describeconfigs_response::DescribeConfigsResponse;
use kafka_protocol::protocol_responses::describeconfigs_response::Resource as ResponseResource;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use state::*;
use std::cell::RefCell;
use std::cell::RefMut;
use std::io::stdout;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use tcp_stream_util;
use tcp_stream_util::TcpRequestError;
use termion::screen::AlternateScreen;
use user_interface::ui;

pub struct BootstrapServer(pub String);

pub enum MoveSelection { Up, Down, Top, Bottom, SearchNext }

pub enum TopicQuery { NoQuery, Query(String) }

pub enum Message {
    GetTopics(BootstrapServer),
    SelectTopic(MoveSelection),
    SetTopicQuery(TopicQuery),
    DeleteTopic(BootstrapServer),
    ToggleTopicInfo(BootstrapServer),
}

enum Event {
    Error(String),
    ListTopics(Response<MetadataResponse>),
    TopicSelected(fn(&State) -> usize),
    TopicQuerySet(Option<String>),
    TopicDeleted(Box<Fn(&State) -> Option<String>>),
    InfoToggled(Box<Fn(&State) -> Option<TopicInfoState>>),
}

pub fn start() -> Sender<Message> {
    let (sender, receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    thread::spawn(move || {
        let screen = &mut AlternateScreen::from(stdout()); // a single alternate screen for all UI to write to
        let state = RefCell::new(State::new()); // RefCell for interior mutability ('unsafe' code)

        for message in receiver {
            if let Some(updated_state) = update_state(to_event(message), state.borrow_mut()) {
                state.swap(&RefCell::new(updated_state));
            }
            ui::update_with_state(&state.borrow(), screen);
        }
    });

    sender
}

fn to_event(message: Message) -> Event {
    match message {
        GetTopics(BootstrapServer(bootstrap)) => {
            let result: Result<Response<MetadataResponse>, TcpRequestError> =
                tcp_stream_util::request(
                    bootstrap,
                    Request::of(MetadataRequest { topics: None, allow_auto_topic_creation: false }, 3, 5),
                );
            match result {
                Ok(response) => ListTopics(response),
                Err(e) => {
                    eprintln!("{}", e.error);
                    Error(e.error)
                }
            }
        }

        SelectTopic(direction) => {
            match direction {
                Up => TopicSelected(|state: &State| {
                    if state.selected_index > 0 { state.selected_index - 1 } else { state.selected_index }
                }),
                Down => TopicSelected(|state: &State| {
                    match state.metadata {
                        Some(ref metadata) => if state.selected_index < (metadata.topic_metadata.len() - 1) { state.selected_index + 1 } else { state.selected_index },
                        None => 0
                    }
                }),
                Top => TopicSelected(|state: &State| 0),
                Bottom => TopicSelected(|state: &State| {
                    state.metadata.as_ref().map(|metadata| {
                        metadata.topic_metadata.len() - 1
                    }).unwrap_or(0)
                }),
                SearchNext => TopicSelected(|state: &State| state.find_next_index(false).unwrap_or(state.selected_index))
            }
        }

        SetTopicQuery(query) => {
            match query {
                Query(q) => TopicQuerySet(Some(q)),
                NoQuery => TopicQuerySet(None)
            }
        }

        DeleteTopic(BootstrapServer(bootstrap)) => {
            TopicDeleted(Box::from(move |state: &State| {
                state.metadata.as_ref().and_then(|metadata| {
                    metadata.topic_metadata.get(state.selected_index).and_then(|delete_topic_metadata| {
                        let delete_topic_name = delete_topic_metadata.topic.clone();
                        let result: Result<Response<DeleteTopicsResponse>, TcpRequestError> =
                            tcp_stream_util::request(
                                bootstrap.clone(),
                                Request::of(DeleteTopicsRequest { topics: vec![delete_topic_name.clone()], timeout: 30_000 }, 20, 1),
                            );

                        match result {
                            Ok(response) => {
                                if response.response_message.topic_error_codes.iter().all(|err| err.error_code == 0) {
                                    Some(delete_topic_name)
                                } else {
                                    None
                                }
                            }
                            Err(err) => {
                                eprintln!("{}", err.error);
                                None
                            }
                        }
                    })
                })
            }))
        }

        ToggleTopicInfo(BootstrapServer(bootstrap)) => {
            InfoToggled(Box::from(move |state: &State| {
                match state.topic_info_state {
                    Some(_) => None,
                    None => {
                        let resource = Resource {
                            resource_type: ResourceTypes::Topic as i8,
                            resource_name: state.selected_topic_name().expect("failure to get topic name"),
                            config_names: None,
                        };
                        let result: Result<Response<DescribeConfigsResponse>, TcpRequestError> =
                            tcp_stream_util::request(
                                bootstrap.clone(),
                                Request::of(DescribeConfigsRequest { resources: vec![resource], include_synonyms: false }, 32, 1),
                            );
                        match result {
                            Ok(response) => {
                                state.selected_topic_metadata().and_then(|topic_metadata| {
                                    let resource = response.response_message.resources
                                        .into_iter()
                                        .filter(|resource| resource.resource_name.eq(&state.selected_topic_name().unwrap()))
                                        .collect::<Vec<ResponseResource>>();

                                    match resource.first() {
                                        None => None,
                                        Some(resource) => Some(TopicInfoState { topic_metadata, config_resource: resource.clone() })
                                    }
                                })
                            }
                            Err(err) => {
                                eprintln!("{}", err.error);
                                None
                            }
                        }
                    }
                }
            }))
        }
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
        TopicQuerySet(query) => {
            current_state.topic_name_query = query;
            Some(current_state.clone())
        }
        TopicDeleted(boxed_delete_fn) => {
            let deleted_name = boxed_delete_fn(&current_state);
            match deleted_name {
                None => None,
                Some(deleted_name) => {
                    current_state.marked_deleted.push(deleted_name);
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

