use AppConfig;
use event_bus::Event::*;
use event_bus::Message::*;
use event_bus::MoveSelection::*;
use event_bus::TopicQuery::*;
use kafka_protocol::protocol_request::Request;
use kafka_protocol::protocol_requests::*;
use kafka_protocol::protocol_response::Response;
use kafka_protocol::protocol_responses::*;
use state::*;
use state::CurrentView;
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

pub struct ConsumerGroup(pub String);

pub enum MoveSelection { Up, Down, Top, Bottom, SearchNext }

pub enum TopicQuery { NoQuery, Query(String) }

pub enum Message {
    Noop,
    UserInput(String),
    GetTopics(BootstrapServer),
    SelectTopic(MoveSelection),
    SetTopicQuery(TopicQuery),
    DeleteTopic(BootstrapServer),
    ToggleTopicInfo(BootstrapServer),
    TogglePartitionInfo(BootstrapServer, Option<ConsumerGroup>),
}

enum Event {
    Error(String),
    StateIdentity,
    UserInputUpdated(String),
    ListTopics(Response<metadata_response::MetadataResponse>),
    TopicSelected(fn(&State) -> usize),
    TopicQuerySet(Option<String>),
    TopicDeleted(Box<Fn(&State) -> Option<String>>),
    InfoToggled(Box<Fn(&State) -> Option<TopicInfoState>>),
    PartitionsToggled(Box<Fn(&State) -> Option<PartitionInfoState>>),
}

pub fn start() -> Sender<Message> {
    let (sender, receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    thread::spawn(move || {
        let state = RefCell::new(State::new()); // RefCell for interior mutability ('unsafe' code)

        for message in receiver {
            if let Some(updated_state) = update_state(to_event(message), state.borrow_mut()) {
                state.swap(&RefCell::new(updated_state));
            }
            ui::update_with_state(&state.borrow());
        }
    });

    sender
}

fn to_event(message: Message) -> Event {
    match message {
        Noop => StateIdentity,

        UserInput(input) => UserInputUpdated(input),

        GetTopics(BootstrapServer(bootstrap)) => {
            let result: Result<Response<metadata_response::MetadataResponse>, TcpRequestError> =
                tcp_stream_util::request(
                    bootstrap,
                    Request::of(metadata_request::MetadataRequest { topics: None, allow_auto_topic_creation: false }, 3, 5),
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
                        let result: Result<Response<deletetopics_response::DeleteTopicsResponse>, TcpRequestError> =
                            tcp_stream_util::request(
                                bootstrap.clone(),
                                Request::of(deletetopics_request::DeleteTopicsRequest { topics: vec![delete_topic_name.clone()], timeout: 30_000 }, 20, 1),
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
                        let resource = describeconfigs_request::Resource {
                            resource_type: describeconfigs_request::ResourceTypes::Topic as i8,
                            resource_name: state.selected_topic_name().expect("failure to get topic name"),
                            config_names: None,
                        };
                        let result: Result<Response<describeconfigs_response::DescribeConfigsResponse>, TcpRequestError> =
                            tcp_stream_util::request(
                                bootstrap.clone(),
                                Request::of(describeconfigs_request::DescribeConfigsRequest { resources: vec![resource], include_synonyms: false }, 32, 1),
                            );
                        match result {
                            Ok(response) => {
                                state.selected_topic_metadata().and_then(|topic_metadata| {
                                    let resource = response.response_message.resources
                                        .into_iter()
                                        .filter(|resource| resource.resource_name.eq(&state.selected_topic_name().unwrap()))
                                        .collect::<Vec<describeconfigs_response::Resource>>();

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

        TogglePartitionInfo(BootstrapServer(bootstrap), opt_consumer_group) => {
            PartitionsToggled(Box::from(move |state: &State| {
                match state.partition_info_state {
                    Some(_) => None,
                    None => {
                        match opt_consumer_group {
                            None => None,
                            Some(ConsumerGroup(ref group_id)) => {
                                let topic = state.selected_topic_metadata().map(|metadata| {
                                    offsetfetch_request::Topic { topic: metadata.topic.clone(), partitions: metadata.partition_metadata.iter().map(|p| p.partition).collect() }
                                });
                                topic.and_then(|topic| {
                                    let result: Result<Response<offsetfetch_response::OffsetFetchResponse>, TcpRequestError> =
                                        tcp_stream_util::request(
                                            bootstrap.clone(),
                                            Request::of(offsetfetch_request::OffsetFetchRequest { group_id: group_id.clone(), topics: vec![topic.clone()] }, 9, 3),
                                        );
                                    let partition_responses =
                                        result.and_then(|result| {
                                            let responses = result.response_message.responses
                                                .into_iter()
                                                .find(|response| response.topic.eq(&topic.topic))
                                                .map(|response| response.partition_responses);
                                            match responses {
                                                None => Err(TcpRequestError::from("Topic not returned from API request")),
                                                Some(partition_responses) => Ok(partition_responses)
                                            }
                                        });
                                    match partition_responses {
                                        Err(err) => {
                                            eprintln!("{}", err.error);
                                            None
                                        }
                                        Ok(partition_responses) => {
                                            Some(PartitionInfoState { selected_index: 0, partition_offsets: partition_responses })
                                        }
                                    }
                                })
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
        StateIdentity => Some(current_state.clone()),
        UserInputUpdated(input) => {
            current_state.user_input = if !input.is_empty() { Some(input) } else { None };
            Some(current_state.clone())
        }
        ListTopics(response) => {
            current_state.metadata = Some(response.response_message);
            current_state.marked_deleted = vec![];
            current_state.current_view = CurrentView::Topics;
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
            current_state.current_view = match current_state.topic_info_state {
                Some(_) => CurrentView::TopicInfo,
                None => CurrentView::Topics
            };
            Some(current_state.clone())
        }
        PartitionsToggled(partition_info_fn) => {
            current_state.partition_info_state = partition_info_fn(&current_state);
            current_state.current_view = if current_state.partition_info_state.is_some() { CurrentView::Partitions } else { CurrentView::Topics };
            Some(current_state.clone())
        }
    }
}

