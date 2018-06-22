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
use utils;

pub struct BootstrapServer(pub String);

pub struct ConsumerGroup(pub String);

pub enum MoveSelection { Up, Down, Top, Bottom, SearchNext }

pub enum TopicQuery { NoQuery, Query(String) }

pub enum Message {
    Noop,
    UserInput(String),
    GetTopics(BootstrapServer),
    Select(MoveSelection),
    SetTopicQuery(TopicQuery),
    DeleteTopic(BootstrapServer),
    ToggleTopicInfo(BootstrapServer),
    TogglePartitionInfo(BootstrapServer, Option<ConsumerGroup>),
}

enum Event {
    StateIdentity,
    UserInputUpdated(String),
    ListTopics(StateFn<metadata_response::MetadataResponse>),
    SelectionUpdated(StateFn<(CurrentView, usize)>),
    TopicQuerySet(Option<String>),
    TopicDeleted(StateFn<String>),
    InfoToggled(StateFn<Option<TopicInfoState>>),
    PartitionsToggled(StateFn<Option<PartitionInfoState>>),
}

pub fn start() -> Sender<Message> {
    let (sender, receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    thread::spawn(move || {
        let state = RefCell::new(State::new()); // RefCell for interior mutability ('unsafe' code)

        for message in receiver {
            match update_state(to_event(message), state.borrow_mut()) {
                Ok(updated_state) => state.swap(&RefCell::new(updated_state)),
                Err(StateFNError::Error(error)) => panic!(error),
                Err(StateFNError::Caused(error, cause)) => panic!(error)
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
                    .map_err(|err| StateFNError::caused("Error encountered trying to retrieve topics", err))
            }))
        }

        Select(direction) => {
            SelectionUpdated(Box::from(move |state: &State| {
                match state.current_view {
                    CurrentView::Topics => {
                        let selected_index =
                            match direction {
                                Up => if state.selected_index > 0 { state.selected_index - 1 } else { state.selected_index },
                                Down => {
                                    match state.metadata {
                                        Some(ref metadata) => if state.selected_index < (metadata.topic_metadata.len() - 1) { state.selected_index + 1 } else { state.selected_index },
                                        None => 0
                                    }
                                }
                                Top => 0,
                                Bottom => {
                                    state.metadata.as_ref().map(|metadata| {
                                        metadata.topic_metadata.len() - 1
                                    }).unwrap_or(0)
                                }
                                SearchNext => state.find_next_index(false).unwrap_or(state.selected_index)
                            };
                        Ok((CurrentView::Topics, selected_index))
                    }
                    CurrentView::Partitions => {
                        let selected_index =
                            state.partition_info_state.as_ref().map(|partition_info_state| {
                                let selected_index = partition_info_state.selected_index;
                                match direction {
                                    Up => if selected_index > 0 { selected_index - 1 } else { selected_index },
                                    Down => if selected_index < (partition_info_state.partition_metadata.len() - 1) { selected_index + 1 } else { selected_index },
                                    Top => 0,
                                    Bottom => partition_info_state.partition_metadata.len() - 1,
                                    SearchNext => selected_index // not implemented
                                }
                            }).unwrap_or(0);
                        Ok((CurrentView::Partitions, selected_index))
                    }
                    CurrentView::TopicInfo => Ok((CurrentView::TopicInfo, 0)) // not implemented
                }
            }))
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
                                    Err(StateFNError::caused("Failed to delete topic", TcpRequestError::of(format!("Non-zero topic error code encountered: {:?}", map))))
                                }
                            }
                            Err(err) => Err(StateFNError::caused("Failed to delete topic", err))
                        }
                    }).unwrap_or(Err(StateFNError::error("Could not select or find topic to delete")))
                }).unwrap_or(Err(StateFNError::error("Topic metadata not available")))
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
                            .map_err(|err| StateFNError::caused("DescribeConfigs request failed", err))
                            .and_then(|response| {
                                state.selected_topic_metadata().map(|topic_metadata| {
                                    let resource = response.response_message.resources
                                        .into_iter()
                                        .filter(|resource| resource.resource_name.eq(&state.selected_topic_name().unwrap()))
                                        .collect::<Vec<describeconfigs_response::Resource>>();

                                    match resource.first() {
                                        None => Err(StateFNError::caused("", TcpRequestError::from("API response missing topic resource info"))),
                                        Some(resource) => Ok(Some(TopicInfoState { topic_metadata, config_resource: resource.clone() }))
                                    }
                                }).unwrap_or(Err(StateFNError::error("Could not select or find topic metadata")))
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
                        state.metadata.as_ref()
                            .and_then(|metadata| metadata.topic_metadata.get(state.selected_index))
                            .map(|topic_metadata: &metadata_response::TopicMetadata| &topic_metadata.partition_metadata)
                            .map(|partition_metadata| {
                                let topic = state.selected_topic_metadata().map(|metadata| {
                                    listoffsets_request::Topic {
                                        topic: metadata.topic.clone(),
                                        partitions: metadata.partition_metadata.iter()
                                            .map(|p| listoffsets_request::Partition { partition: p.partition, timestamp: -1 }).collect(),
                                    }
                                });

                                let partition_responses =
                                    topic.map(|topic| {
                                        let listoffsets_response: Result<Response<listoffsets_response::ListOffsetsResponse>, TcpRequestError> =
                                            tcp_stream_util::request(
                                                bootstrap.clone(),
                                                Request::of(listoffsets_request::ListOffsetsRequest { replica_id: -1, isolation_level: 0, topics: vec![topic] }, 2, 2),
                                            );
                                        listoffsets_response.map_err(|err| StateFNError::caused("ListOffsets request failed", err))
                                    }).unwrap_or(Err(StateFNError::error("Could not select or find topic metadata")))
                                        .and_then(|response| {
                                            match response.response_message.responses.first() {
                                                Some(topic_offsets) => Ok(topic_offsets.partition_responses.clone()),
                                                None => Err(StateFNError::error("ListOffsets API did not return any topic offsets"))
                                            }
                                        });

                                partition_responses.and_then(|partition_responses| {
                                    let partition_offsets = partition_responses.into_iter().map(|partition_response| {
                                        (partition_response.partition, partition_response)
                                    }).collect::<HashMap<i32, listoffsets_response::PartitionResponse>>();

                                    match opt_consumer_group {
                                        None => Ok(Some(PartitionInfoState { selected_index: 0, partition_metadata: partition_metadata.clone(), partition_offsets, consumer_offsets: HashMap::new() })),
                                        Some(ConsumerGroup(ref group_id)) => {
                                            let topic = state.selected_topic_metadata().map(|metadata| {
                                                offsetfetch_request::Topic {
                                                    topic: metadata.topic.clone(),
                                                    partitions: metadata.partition_metadata.iter().map(|p| p.partition).collect(),
                                                }
                                            });
                                            topic.map(|topic| {
                                                let offsetfetch_result: Result<Response<offsetfetch_response::OffsetFetchResponse>, TcpRequestError> =
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
                                                    offsetfetch_result.and_then(|result| {
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
                                                    .map(|partition_responses| {
                                                        partition_responses.into_iter().map(|p| (p.partition, p)).collect::<HashMap<i32, offsetfetch_response::PartitionResponse>>()
                                                    })
                                                    .map(|consumer_offsets| {
                                                        Some(PartitionInfoState { selected_index: 0, partition_metadata: partition_metadata.clone(), partition_offsets, consumer_offsets })
                                                    })
                                                    .map_err(|err| StateFNError::caused("Error encountered trying to retrieve partition offsets", err))
                                            }).unwrap_or(Err(StateFNError::error("Could not select or find topic metadata")))
                                        }
                                    }
                                })
                            }).unwrap_or(Err(StateFNError::error("Could not select or find partition metadata")))
                    }
                }
            }))
        }
    }
}

fn update_state(event: Event, mut current_state: RefMut<State>) -> Result<State, StateFNError> {
    match event {
        StateIdentity => Ok(current_state.clone()),
        UserInputUpdated(input) => {
            current_state.user_input = if !input.is_empty() { Some(input) } else { None };
            Ok(current_state.clone())
        }
        ListTopics(get_metadata) => {
            get_metadata(&current_state).map(|response: metadata_response::MetadataResponse| {
                current_state.metadata = Some(response);
                current_state.marked_deleted = vec![];
                current_state.current_view = CurrentView::Topics;
                current_state.clone()
            })
        }
        SelectionUpdated(select_fn) => {
            match select_fn(&current_state) {
                Err(e) => Err(e),
                Ok((CurrentView::Topics, selected_index)) => {
                    current_state.selected_index = selected_index;
                    Ok(current_state.clone())
                }
                Ok((CurrentView::Partitions, selected_index)) => {
                    current_state.partition_info_state = current_state.partition_info_state.as_mut().map(|state| {
                        state.selected_index = selected_index;
                        state.clone()
                    });
                    Ok(current_state.clone())
                }
                Ok((CurrentView::TopicInfo, _)) => Ok(current_state.clone())
            }
        }
        TopicQuerySet(query) => {
            current_state.topic_name_query = query;
            Ok(current_state.clone())
        }
        TopicDeleted(delete_fn) => {
            delete_fn(&current_state).map(|deleted_name: String| {
                current_state.marked_deleted.push(deleted_name);
                current_state.clone()
            })
        }
        InfoToggled(toggle_fn) => {
            toggle_fn(&current_state).map(|topic_info_state: Option<TopicInfoState>| {
                current_state.topic_info_state = topic_info_state;
                current_state.current_view = match current_state.topic_info_state {
                    Some(_) => CurrentView::TopicInfo,
                    None => CurrentView::Topics
                };
                current_state.clone()
            })
        }
        PartitionsToggled(partition_info_fn) => {
            partition_info_fn(&current_state).map(|partition_info_state: Option<PartitionInfoState>| {
                current_state.partition_info_state = partition_info_state;
                current_state.current_view = if current_state.partition_info_state.is_some() { CurrentView::Partitions } else { CurrentView::Topics };
                current_state.clone()
            })
        }
    }
}

