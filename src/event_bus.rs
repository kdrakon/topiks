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
use std::collections::HashMap;
use std::fmt::Display;
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
    StateIdentity,
    UserInputUpdated(String),
    ListTopics(StateFn<metadata_response::MetadataResponse, TcpRequestError>),
    TopicSelected(fn(&State) -> usize),
    TopicQuerySet(Option<String>),
    TopicDeleted(StateFn<String, TcpRequestError>),
    InfoToggled(StateFn<Option<TopicInfoState>, TcpRequestError>),
    PartitionsToggled(StateFn<Option<PartitionInfoState>, TcpRequestError>),
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
            ListTopics(Box::from(move |state: &State| {
                let result: Result<Response<metadata_response::MetadataResponse>, TcpRequestError> =
                    tcp_stream_util::request(
                        bootstrap.clone(),
                        Request::of(metadata_request::MetadataRequest { topics: None, allow_auto_topic_creation: false }, 3, 5),
                    );
                result
                    .map(|response| response.response_message)
                    .map_err(|err| StateFNError::of("Error encountered trying to retrieve topics", err))
            }))
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
                state.metadata.as_ref().map(|metadata| {
                    metadata.topic_metadata.get(state.selected_index).map(|delete_topic_metadata: &metadata_response::TopicMetadata| {
                        let delete_topic_name = delete_topic_metadata.topic.clone();
                        let result: Result<Response<deletetopics_response::DeleteTopicsResponse>, TcpRequestError> =
                            tcp_stream_util::request(
                                bootstrap.clone(),
                                Request::of(deletetopics_request::DeleteTopicsRequest { topics: vec![delete_topic_name.clone()], timeout: 30_000 }, 20, 1),
                            );
                        match result {
                            Ok(response) => {
                                let map =
                                    response.response_message.topic_error_codes.iter().map(|err| (&err.topic, err.error_code)).collect::<HashMap<&String, i16>>();
                                if map.values().all(|err_code| *err_code == 0) {
                                    Ok(delete_topic_name)
                                } else {
                                    Err(StateFNError::of("Failed to delete topic", TcpRequestError::of(format!("Non-zero topic error code encountered: {:?}", map))))
                                }
                            }
                            Err(err) => Err(StateFNError::of("Failed to delete topic", err))
                        }
                    }).unwrap_or(Err(StateFNError::just("Could not select or find topic to delete")))
                }).unwrap_or(Err(StateFNError::just("Topic metadata not available")))
            }))
        }

        ToggleTopicInfo(BootstrapServer(bootstrap)) => {
            InfoToggled(Box::from(move |state: &State| {
                match state.topic_info_state {
                    Some(_) => Ok(None),
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
                        result
                            .map_err(|err| StateFNError::of("DescribeConfigs request failed", err))
                            .and_then(|response| {
                                state.selected_topic_metadata().map(|topic_metadata| {
                                    let resource = response.response_message.resources
                                        .into_iter()
                                        .filter(|resource| resource.resource_name.eq(&state.selected_topic_name().unwrap()))
                                        .collect::<Vec<describeconfigs_response::Resource>>();

                                    match resource.first() {
                                        None => Err(StateFNError::of("", TcpRequestError::from("API response missing topic resource info"))),
                                        Some(resource) => Ok(Some(TopicInfoState { topic_metadata, config_resource: resource.clone() }))
                                    }
                                }).unwrap_or(Err(StateFNError::just("Could not select or find topic metadata")))
                            })
                    }
                }
            }))
        }

        TogglePartitionInfo(BootstrapServer(bootstrap), opt_consumer_group) => {
            PartitionsToggled(Box::from(move |state: &State| {
                match state.partition_info_state {
                    Some(_) => Ok(None),
                    None => {
                        match opt_consumer_group {
                            None => Ok(Some(PartitionInfoState { selected_index: 0, partition_offsets: vec![] })),
                            Some(ConsumerGroup(ref group_id)) => {
                                let topic = state.selected_topic_metadata().map(|metadata| {
                                    offsetfetch_request::Topic { topic: metadata.topic.clone(), partitions: metadata.partition_metadata.iter().map(|p| p.partition).collect() }
                                });
                                topic.map(|topic| {
                                    let result: Result<Response<offsetfetch_response::OffsetFetchResponse>, TcpRequestError> =
                                        tcp_stream_util::request(
                                            bootstrap.clone(),
                                            Request::of(offsetfetch_request::OffsetFetchRequest { group_id: group_id.clone(), topics: vec![topic.clone()] }, 9, 3),
                                        ).and_then(|result: Response<offsetfetch_response::OffsetFetchResponse>| {
                                            if result.response_message.error_code != 0 {
                                                Err(TcpRequestError::of(format!("Error code {}", result.response_message.error_code)))
                                            } else {
                                                Ok(result)
                                            }
                                        });
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
                                    partition_responses
                                        .map(|partition_responses| Some(PartitionInfoState { selected_index: 0, partition_offsets: partition_responses }))
                                        .map_err(|err| StateFNError::of("Error encountered trying to retrieve partition offsets", err))
                                }).unwrap_or(Err(StateFNError::just("Could not select or find topic metadata")))
                            }
                        }
                    }
                }
            }))
        }
    }
}

fn update_state(event: Event, mut current_state: RefMut<State>) -> Option<State> { // TODO change to Result
    match event {
        StateIdentity => Some(current_state.clone()),
        UserInputUpdated(input) => {
            current_state.user_input = if !input.is_empty() { Some(input) } else { None };
            Some(current_state.clone())
        }
        ListTopics(get_metadata) => {
            match get_metadata(&current_state) {
                Err(e) => None, // TODO
                Ok(response) => {
                    current_state.metadata = Some(response);
                    current_state.marked_deleted = vec![];
                    current_state.current_view = CurrentView::Topics;
                    Some(current_state.clone())
                }
            }
        }
        TopicSelected(select_fn) => {
            current_state.selected_index = select_fn(&current_state);
            Some(current_state.clone())
        }
        TopicQuerySet(query) => {
            current_state.topic_name_query = query;
            Some(current_state.clone())
        }
        TopicDeleted(delete_fn) => {
            match delete_fn(&current_state) {
                Err(e) => None, // TODO
                Ok(deleted_name) => {
                    current_state.marked_deleted.push(deleted_name);
                    Some(current_state.clone())
                }
            }
        }
        InfoToggled(toggle_fn) => {
            match toggle_fn(&current_state) {
                Err(e) => None, // TODO
                Ok(topic_info_state) => {
                    current_state.topic_info_state = topic_info_state;
                    current_state.current_view = match current_state.topic_info_state {
                        Some(_) => CurrentView::TopicInfo,
                        None => CurrentView::Topics
                    };
                    Some(current_state.clone())
                }
            }
        }
        PartitionsToggled(partition_info_fn) => {
            match partition_info_fn(&current_state) {
                Err(e) => None, // TODO
                Ok(partition_info_state) => {
                    current_state.partition_info_state = partition_info_state;
                    current_state.current_view = if current_state.partition_info_state.is_some() { CurrentView::Partitions } else { CurrentView::Topics };
                    Some(current_state.clone())
                }
            }
        }
    }
}

