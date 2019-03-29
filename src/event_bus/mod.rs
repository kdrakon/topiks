use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;

use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

use crate::api_client::ApiClient;
use crate::api_client::ApiClientProvider;
use crate::api_client::ApiClientTrait;
use crate::api_client::ApiRequestError;
use crate::event_bus::Event::*;
use crate::event_bus::Message::Delete;
use crate::event_bus::Message::*;
use crate::event_bus::MoveSelection::*;
use crate::event_bus::TopicQuery::*;
use crate::kafka_protocol::protocol_request::Request;
use crate::kafka_protocol::protocol_requests;
use crate::kafka_protocol::protocol_requests::*;
use crate::kafka_protocol::protocol_response::Response;
use crate::kafka_protocol::protocol_responses::findcoordinator_response::Coordinator;
use crate::kafka_protocol::protocol_responses::*;
use crate::state::CurrentView;
use crate::state::*;
use crate::user_interface::ui;
use crate::util::utils::{controller_broker, Flatten};
use crate::KafkaServerAddr;
use crate::IO;

#[derive(Clone)]
pub struct ConsumerGroup(pub String, pub findcoordinator_response::Coordinator);

const PAGE_MOVEMENT: i32 = 10;

pub enum MoveSelection {
    Up,
    PageUp,
    Down,
    PageDown,
    Top,
    Bottom,
    SearchNext,
}

pub enum TopicQuery {
    NoQuery,
    Query(String),
}

#[derive(Clone, Debug)]
pub enum Creation {
    Topic { name: String, partitions: i32, replication_factor: i16 },
}

pub enum Deletion {
    Topic(String),
    Config(String),
}

pub enum Modification {
    Config(String),
}

pub enum MetadataPayload {
    Metadata(metadata_response::MetadataResponse),
    PartitionsMetadata(
        metadata_response::MetadataResponse,
        Vec<metadata_response::PartitionMetadata>,
        HashMap<i32, listoffsets_response::PartitionResponse>,
        Option<HashMap<i32, offsetfetch_response::PartitionResponse>>,
    ),
    TopicInfoMetadata(metadata_response::MetadataResponse, describeconfigs_response::Resource),
}

pub enum Message {
    Quit,
    Noop,
    GetMetadata(KafkaServerAddr, Option<ConsumerGroup>),
    ToggleView(CurrentView),
    DisplayUIMessage(DialogMessage),
    UserInput(String),
    Select(MoveSelection),
    SetTopicQuery(TopicQuery),
    Create(KafkaServerAddr, Creation, i32),
    Delete(KafkaServerAddr, i32),
    ModifyValue(KafkaServerAddr, Option<String>),
}

enum Event {
    Exiting,
    StateIdentity,
    MetadataRetrieved(StateFn<MetadataPayload>),
    ViewToggled(CurrentView),
    ShowUIMessage(DialogMessage),
    UserInputUpdated(String),
    SelectionUpdated(StateFn<(CurrentView, usize)>),
    TopicQuerySet(Option<String>),
    ResourceCreated(StateFn<String>),
    ResourceDeleted(StateFn<Deletion>),
    ValueModified(StateFn<Modification>),
}

pub fn start() -> Sender<Message> {
    let (sender, receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();
    let thread_sender = sender.clone();

    thread::spawn(move || {
        let state = RefCell::new(State::new()); // RefCell for interior mutability
        let screen = &mut AlternateScreen::from(stdout().into_raw_mode().unwrap());

        for message in receiver {
            match to_event(message, Box::new(|| IO::new(Box::new(|| Ok(ApiClient::new()))))) {
                Exiting => break,
                non_exit_event => {
                    match update_state(non_exit_event, state.borrow_mut()) {
                        Ok(updated_state) => state.swap(&RefCell::new(updated_state)),
                        Err(StateFNError::Error(error)) => {
                            thread_sender.send(Message::DisplayUIMessage(DialogMessage::Error(error))).unwrap();
                        }
                        Err(StateFNError::Caused(error, cause)) => {
                            thread_sender.send(Message::DisplayUIMessage(DialogMessage::Error(format!("{}: {}", error, cause)))).unwrap();
                        }
                    }
                    ui::update_with_state(&state.borrow(), screen);
                }
            }
        }

        screen.flush().unwrap(); // final flush before handing screen back to shell
    });

    sender
}

fn to_event<T: ApiClientTrait + 'static>(message: Message, api_client_provider: ApiClientProvider<T>) -> Event {
    match message {
        Quit => Exiting,
        Noop => StateIdentity,
        DisplayUIMessage(message) => ShowUIMessage(message),
        UserInput(input) => UserInputUpdated(input),
        ToggleView(view) => ViewToggled(view),

        GetMetadata(bootstrap_server, opt_consumer_group) => MetadataRetrieved(Box::from(move |state: &State| {
            let metadata_response = retrieve_metadata(api_client_provider(), &bootstrap_server)
                .into_result()
                .map_err(|err| StateFNError::caused("Error encountered trying to retrieve topics", err));

            match state.current_view {
                CurrentView::Topics | CurrentView::HelpScreen => {
                    metadata_response.map(|metadata_response| MetadataPayload::Metadata(metadata_response))
                }
                CurrentView::Partitions => metadata_response.and_then(|metadata_response| {
                    state
                        .selected_topic_metadata()
                        .map(|topic_metadata| {
                            retrieve_partition_metadata_and_offsets(api_client_provider(), &bootstrap_server, &metadata_response, &topic_metadata)
                                .into_result()
                                .and_then(|(partition_metadata, partition_offsets)| match opt_consumer_group {
                                    None => Ok(MetadataPayload::PartitionsMetadata(metadata_response, partition_metadata, partition_offsets, None)),
                                    Some(ConsumerGroup(ref group_id, ref coordinator)) => retrieve_consumer_offsets(
                                        api_client_provider(),
                                        group_id,
                                        coordinator,
                                        &topic_metadata,
                                        bootstrap_server.use_tls,
                                    )
                                    .into_result()
                                    .map(|consumer_offsets| {
                                        MetadataPayload::PartitionsMetadata(
                                            metadata_response,
                                            partition_metadata,
                                            partition_offsets,
                                            Some(consumer_offsets),
                                        )
                                    }),
                                })
                                .map_err(|err| StateFNError::caused("Error retrieving partition metadata", err))
                        })
                        .unwrap_or(Err(StateFNError::error("Could not find selected topic metadata")))
                }),
                CurrentView::TopicInfo => metadata_response.and_then(|metadata_response| {
                    state
                        .selected_topic_name()
                        .map(|topic_name| {
                            retrieve_topic_metadata(api_client_provider(), &bootstrap_server, &topic_name)
                                .into_result()
                                .map_err(|err| StateFNError::caused("Error retrieving topic config", err))
                                .map(|resource| MetadataPayload::TopicInfoMetadata(metadata_response, resource))
                        })
                        .unwrap_or(Err(StateFNError::error("No topic selected")))
                }),
            }
        })),

        Select(direction) => {
            SelectionUpdated(Box::from(move |state: &State| {
                match state.current_view {
                    CurrentView::HelpScreen => Ok((CurrentView::HelpScreen, 0)),
                    CurrentView::Topics => {
                        let selected_index = match direction {
                            Up => {
                                if state.selected_index > 0 {
                                    state.selected_index - 1
                                } else {
                                    state.selected_index
                                }
                            }
                            PageUp => {
                                if (state.selected_index as i32 - PAGE_MOVEMENT) <= 0 {
                                    state.selected_index
                                } else {
                                    state.selected_index - (PAGE_MOVEMENT as usize)
                                }
                            }
                            Down => match state.metadata {
                                Some(ref metadata) => {
                                    if state.selected_index < (metadata.topic_metadata.len() - 1) {
                                        state.selected_index + 1
                                    } else {
                                        state.selected_index
                                    }
                                }
                                None => 0,
                            },
                            PageDown => match state.metadata {
                                Some(ref metadata) => {
                                    if ((state.selected_index as i32) + PAGE_MOVEMENT) < (metadata.topic_metadata.len() as i32 - 1) {
                                        state.selected_index + (PAGE_MOVEMENT as usize)
                                    } else {
                                        state.selected_index
                                    }
                                }
                                None => 0,
                            },
                            Top => 0,
                            Bottom => state.metadata.as_ref().map(|metadata| metadata.topic_metadata.len() - 1).unwrap_or(0),
                            SearchNext => state.find_next_index(false).unwrap_or(state.selected_index),
                        };
                        Ok((CurrentView::Topics, selected_index))
                    }
                    CurrentView::Partitions => {
                        let selected_index = state
                            .partition_info_state
                            .as_ref()
                            .map(|partition_info_state| {
                                let selected_index = partition_info_state.selected_index;
                                let entries_len = partition_info_state.partition_metadata.len() - 1;
                                match direction {
                                    Up => {
                                        if selected_index > 0 {
                                            selected_index - 1
                                        } else {
                                            selected_index
                                        }
                                    }
                                    Down => {
                                        if selected_index < entries_len {
                                            selected_index + 1
                                        } else {
                                            selected_index
                                        }
                                    }
                                    PageUp => selected_index,   // not implemented
                                    PageDown => selected_index, // not implemented
                                    Top => 0,
                                    Bottom => partition_info_state.partition_metadata.len() - 1,
                                    SearchNext => selected_index, // not implemented
                                }
                            })
                            .unwrap_or(0);
                        Ok((CurrentView::Partitions, selected_index))
                    }
                    CurrentView::TopicInfo => {
                        let selected_index = state
                            .topic_info_state
                            .as_ref()
                            .map(|topic_info_state| {
                                let selected_index = topic_info_state.selected_index;
                                let entries_len = topic_info_state.config_resource.config_entries.len() - 1;
                                match direction {
                                    Up => {
                                        if selected_index > 0 {
                                            selected_index - 1
                                        } else {
                                            selected_index
                                        }
                                    }
                                    Down => {
                                        if selected_index < entries_len {
                                            selected_index + 1
                                        } else {
                                            selected_index
                                        }
                                    }
                                    PageUp => selected_index,   // not implemented
                                    PageDown => selected_index, // not implemented
                                    Top => 0,
                                    Bottom => entries_len,
                                    SearchNext => selected_index, // not implemented
                                }
                            })
                            .unwrap_or(0);
                        Ok((CurrentView::TopicInfo, selected_index))
                    }
                }
            }))
        }

        SetTopicQuery(query) => match query {
            Query(q) => TopicQuerySet(Some(q)),
            NoQuery => TopicQuerySet(None),
        },

        Create(bootstrap_sever, creation, request_timeout_ms) => ResourceCreated(Box::from(move |state: &State| match &creation {
            Creation::Topic { name, partitions, replication_factor } => {
                if state.current_view != CurrentView::Topics {
                    Err(StateFNError::error("Cannot create topic here"))
                } else {
                    state
                        .metadata
                        .as_ref()
                        .map(|metadata| {
                            controller_broker(metadata)
                                .map(|controller| {
                                    create_topic(
                                        api_client_provider(),
                                        name.clone(),
                                        *partitions,
                                        *replication_factor,
                                        controller,
                                        bootstrap_sever.use_tls,
                                        request_timeout_ms,
                                    )
                                    .into_result()
                                    .map_err(|err| StateFNError::caused("Error creating topic", err))
                                    .and_then(|response| {
                                        let error_message = response
                                            .response_message
                                            .topic_errors
                                            .into_iter()
                                            .filter(|err| err.error_code != 0)
                                            .collect::<Vec<_>>()
                                            .first()
                                            .map(|error| {
                                                format!(
                                                    "({}) {}",
                                                    error.error_code,
                                                    error.error_message.clone().unwrap_or("Unknown error".to_string())
                                                )
                                            });
                                        match error_message {
                                            Some(ref err) => Err(StateFNError::error(err)),
                                            None => Ok(name.clone()),
                                        }
                                    })
                                })
                                .unwrap_or(Err(StateFNError::error("Could not find Kafka controller host from Metadata")))
                        })
                        .unwrap_or(Err(StateFNError::error("Topic metadata not available")))
                }
            }
        })),

        Delete(bootstrap_server, request_timeout_ms) => ResourceDeleted(Box::from(move |state: &State| match state.current_view {
            CurrentView::Partitions => Err(StateFNError::error("Partition deletion not supported")),
            CurrentView::HelpScreen => Err(StateFNError::error("You can't delete this...")),
            CurrentView::Topics => state
                .metadata
                .as_ref()
                .map(|metadata| {
                    metadata
                        .topic_metadata
                        .get(state.selected_index)
                        .map(|delete_topic_metadata: &metadata_response::TopicMetadata| {
                            if delete_topic_metadata.is_internal {
                                Err(StateFNError::error("Can not delete internal topics"))
                            } else {
                                controller_broker(&metadata)
                                    .map(|controller_broker| {
                                        let delete_topic_name = delete_topic_metadata.topic.clone();
                                        let result: Result<Response<deletetopics_response::DeleteTopicsResponse>, ApiRequestError> = delete_topic(
                                            api_client_provider(),
                                            &delete_topic_name,
                                            controller_broker,
                                            bootstrap_server.use_tls,
                                            request_timeout_ms,
                                        )
                                        .into_result();
                                        match result {
                                            Ok(response) => {
                                                let map = response
                                                    .response_message
                                                    .topic_error_codes
                                                    .iter()
                                                    .map(|err| (&err.topic, err.error_code))
                                                    .collect::<HashMap<&String, i16>>();
                                                if map.values().all(|err_code| *err_code == 0) {
                                                    Ok(Deletion::Topic(delete_topic_name))
                                                } else {
                                                    Err(StateFNError::caused(
                                                        "Failed to delete topic",
                                                        ApiRequestError::of(format!("Non-zero topic error code encountered: {:?}", map)),
                                                    ))
                                                }
                                            }
                                            Err(err) => Err(StateFNError::caused("Failed to delete topic", err)),
                                        }
                                    })
                                    .unwrap_or(Err(StateFNError::error("Could not find Kafka controller host from Metadata")))
                            }
                        })
                        .unwrap_or(Err(StateFNError::error("Could not select or find topic to delete")))
                })
                .unwrap_or(Err(StateFNError::error("Topic metadata not available"))),
            CurrentView::TopicInfo => state
                .topic_info_state
                .as_ref()
                .map(|topic_info_state| match topic_info_state.config_resource.config_entries.get(topic_info_state.selected_index) {
                    None => Err(StateFNError::error("Error trying to modify selected config")),
                    Some(config_entry) => {
                        let existing_configs = topic_info_state
                            .config_resource
                            .config_entries
                            .iter()
                            .filter(|c| c.config_source == describeconfigs_response::ConfigSource::TopicConfig as i8)
                            .filter(|c| c.config_name != config_entry.config_name)
                            .map(|c| alterconfigs_request::ConfigEntry { config_name: c.config_name.clone(), config_value: c.config_value.clone() })
                            .collect::<Vec<alterconfigs_request::ConfigEntry>>();

                        let resource = alterconfigs_request::Resource {
                            resource_type: protocol_requests::ResourceTypes::Topic as i8,
                            resource_name: topic_info_state.topic_metadata.topic.clone(),
                            config_entries: existing_configs,
                        };

                        alter_config(api_client_provider(), &bootstrap_server, &resource).map(|_| Deletion::Config(config_entry.config_name.clone()))
                    }
                })
                .unwrap_or(Err(StateFNError::error("Topic metadata not available"))),
        })),

        ModifyValue(bootstrap_server, new_value) => ValueModified(Box::from(move |state: &State| match state.current_view {
            CurrentView::Topics => Err(StateFNError::error("Modifications not supported for topics")),
            CurrentView::Partitions => Err(StateFNError::error("Modifications not supported for partitions")),
            CurrentView::HelpScreen => Err(StateFNError::error("You can't modify this...")),
            CurrentView::TopicInfo => state
                .topic_info_state
                .as_ref()
                .map(|topic_info_state| match topic_info_state.config_resource.config_entries.get(topic_info_state.selected_index) {
                    None => Err(StateFNError::error("Error trying to modify selected config")),
                    Some(config_entry) => {
                        let mut existing_configs = topic_info_state
                            .config_resource
                            .config_entries
                            .iter()
                            .filter(|c| c.config_source == describeconfigs_response::ConfigSource::TopicConfig as i8)
                            .filter(|c| c.config_name != config_entry.config_name)
                            .map(|c| alterconfigs_request::ConfigEntry { config_name: c.config_name.clone(), config_value: c.config_value.clone() })
                            .collect::<Vec<alterconfigs_request::ConfigEntry>>();

                        existing_configs.push(alterconfigs_request::ConfigEntry {
                            config_name: config_entry.config_name.clone(),
                            config_value: new_value.clone(),
                        });

                        let resource = alterconfigs_request::Resource {
                            resource_type: protocol_requests::ResourceTypes::Topic as i8,
                            resource_name: topic_info_state.topic_metadata.topic.clone(),
                            config_entries: existing_configs,
                        };

                        alter_config(api_client_provider(), &bootstrap_server, &resource)
                            .map(|_| Modification::Config(config_entry.config_name.clone()))
                    }
                })
                .unwrap_or(Err(StateFNError::error("Topic info not available"))),
        })),
    }
}

fn update_state(event: Event, mut current_state: RefMut<State>) -> Result<State, StateFNError> {
    match event {
        Exiting => Err(StateFNError::error("Invalid State, can not update state from Exiting event")),
        StateIdentity => Ok(current_state.clone()),
        UserInputUpdated(input) => {
            current_state.user_input = if !input.is_empty() { Some(input) } else { None };
            Ok(current_state.clone())
        }
        ShowUIMessage(message) => {
            match message {
                DialogMessage::None => current_state.dialog_message = None,
                _ => current_state.dialog_message = Some(message),
            }
            Ok(current_state.clone())
        }
        MetadataRetrieved(payload_fn) => payload_fn(&current_state).and_then(|payload: MetadataPayload| match payload {
            MetadataPayload::Metadata(metadata_response) => {
                let mut state = State::new();
                state.current_view = CurrentView::Topics;
                state.selected_index = current_state.selected_index;
                state.metadata = Some(metadata_response);
                Ok(state)
            }
            MetadataPayload::PartitionsMetadata(metadata_response, partition_metadata, partition_offsets, consumer_offsets) => {
                current_state.metadata = Some(metadata_response);
                current_state.partition_info_state =
                    Some(PartitionInfoState::new(partition_metadata, partition_offsets, consumer_offsets.unwrap_or(HashMap::new())));
                Ok(current_state.clone())
            }
            MetadataPayload::TopicInfoMetadata(metadata_response, config_resources) => {
                current_state.metadata = Some(metadata_response);
                current_state.topic_info_state =
                    current_state.selected_topic_metadata().map(|topic_metadata| TopicInfoState::new(topic_metadata, config_resources));
                Ok(current_state.clone())
            }
        }),
        ViewToggled(view) => {
            current_state.current_view = view;
            Ok(current_state.clone())
        }
        SelectionUpdated(select_fn) => match select_fn(&current_state) {
            Err(e) => Err(e),
            Ok((CurrentView::HelpScreen, _)) => Ok(current_state.clone()),
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
            Ok((CurrentView::TopicInfo, selected_index)) => {
                current_state.topic_info_state = current_state.topic_info_state.as_mut().map(|state| {
                    state.selected_index = selected_index;
                    state.clone()
                });
                Ok(current_state.clone())
            }
        },
        TopicQuerySet(query) => {
            current_state.topic_name_query = query;
            Ok(current_state.clone())
        }
        ResourceCreated(create_fn) => create_fn(&current_state).map(|new_topic_name| {
            current_state.dialog_message = Some(DialogMessage::Info(format!("Topic '{}' created. Press 'r' to refresh view.", new_topic_name)));
            current_state.clone()
        }),
        ResourceDeleted(delete_fn) => delete_fn(&current_state).map(|deleted: Deletion| match deleted {
            Deletion::Topic(topic) => {
                current_state.marked_deleted.push(topic);
                current_state.clone()
            }
            Deletion::Config(config) => {
                let current_topic_info_state = current_state.topic_info_state.clone();
                current_state.topic_info_state = current_topic_info_state.map(|mut topic_info_state| {
                    topic_info_state.configs_marked_deleted.push(config);
                    topic_info_state
                });
                current_state.clone()
            }
        }),
        ValueModified(modify_fn) => modify_fn(&current_state).map(|modification: Modification| {
            let current_topic_info_state = current_state.topic_info_state.clone();
            current_state.topic_info_state = current_topic_info_state.map(|mut topic_info_state| {
                match modification {
                    Modification::Config(config_name) => topic_info_state.configs_marked_modified.push(config_name),
                }
                topic_info_state
            });
            current_state.clone()
        }),
    }
}

fn retrieve_metadata<T: ApiClientTrait + 'static>(
    client: IO<T, ApiRequestError>,
    bootstrap_server: &KafkaServerAddr,
) -> IO<metadata_response::MetadataResponse, ApiRequestError> {
    let bootstrap_server = bootstrap_server.clone();
    client.and_then_result(Box::new(move |client: T| {
        let result: Result<Response<metadata_response::MetadataResponse>, ApiRequestError> =
            client.request(&bootstrap_server, Request::of(metadata_request::MetadataRequest { topics: None, allow_auto_topic_creation: false }));

        result.map(|response| {
            let mut metadata_response = response.response_message;
            // sort by topic names before returning
            metadata_response.topic_metadata.sort_by(|a, b| a.topic.to_lowercase().cmp(&b.topic.to_lowercase()));
            metadata_response
        })
    }))
}

fn retrieve_topic_metadata<T: ApiClientTrait + 'static>(
    client: IO<T, ApiRequestError>,
    bootstrap_server: &KafkaServerAddr,
    topic_name: &String,
) -> IO<describeconfigs_response::Resource, ApiRequestError> {
    let bootstrap_server = bootstrap_server.clone();
    let topic_name = topic_name.clone();

    client.and_then_result(Box::new(move |client: T| {
        let resource = describeconfigs_request::Resource {
            resource_type: protocol_requests::ResourceTypes::Topic as i8,
            resource_name: topic_name.clone(),
            config_names: None,
        };

        let result: Result<Response<describeconfigs_response::DescribeConfigsResponse>, ApiRequestError> = client.request(
            &bootstrap_server,
            Request::of(describeconfigs_request::DescribeConfigsRequest { resources: vec![resource], include_synonyms: false }),
        );

        result.and_then(|response| {
            let resource = response
                .response_message
                .resources
                .into_iter()
                .filter(|resource| resource.resource_name.eq(&topic_name))
                .collect::<Vec<describeconfigs_response::Resource>>();

            match resource.first() {
                None => Err(ApiRequestError::from("API response missing topic resource info")),
                Some(resource) => {
                    if resource.error_code == 0 {
                        Ok(resource.clone())
                    } else {
                        let error_msg = resource.error_message.clone().unwrap_or(format!(""));
                        Err(ApiRequestError::of(format!("Error describing config. {}", error_msg)))
                    }
                }
            }
        })
    }))
}

fn retrieve_partition_metadata_and_offsets<T: ApiClientTrait + 'static>(
    client: IO<T, ApiRequestError>,
    bootstrap_server: &KafkaServerAddr,
    metadata_response: &metadata_response::MetadataResponse,
    topic_metadata: &metadata_response::TopicMetadata,
) -> IO<(Vec<metadata_response::PartitionMetadata>, HashMap<i32, listoffsets_response::PartitionResponse>), ApiRequestError> {
    let metadata_response = metadata_response.clone();
    let topic_metadata = topic_metadata.clone();
    let bootstrap_server = bootstrap_server.clone();

    client.and_then_result(Box::new(move |client: T| {
        let partition_metadata = &topic_metadata.partition_metadata;
        let broker_id_to_host_map = metadata_response
            .brokers
            .iter()
            .map(|b| (b.node_id, KafkaServerAddr::of(b.host.clone(), b.port, bootstrap_server.use_tls)))
            .collect::<HashMap<i32, KafkaServerAddr>>();

        let mut sorted_partition_metadata = partition_metadata.clone();
        sorted_partition_metadata.sort_by(|a, b| a.partition.cmp(&b.partition));

        let partitions_grouped_by_broker: HashMap<i32, Vec<i32>> = partition_metadata.iter().fold(HashMap::new(), |mut map, partition| {
            let mut partitions = map.get_mut(&partition.leader).map(|vec| vec.clone()).unwrap_or(vec![]);
            partitions.push(partition.partition);
            map.insert(partition.leader, partitions.clone());
            map
        });

        let partition_offset_requests = partitions_grouped_by_broker
            .iter()
            .map(|(broker_id, partitions)| {
                (
                    broker_id_to_host_map.get(broker_id).unwrap_or(&bootstrap_server),
                    listoffsets_request::Topic {
                        topic: topic_metadata.topic.clone(),
                        partitions: partitions.iter().map(|p| listoffsets_request::Partition { partition: *p, timestamp: -1 }).collect(),
                    },
                )
            })
            .collect::<Vec<(&KafkaServerAddr, listoffsets_request::Topic)>>();

        let partition_offset_responses = partition_offset_requests
            .into_iter()
            .map(|(broker_address, topic)| {
                let listoffsets_response: Result<Response<listoffsets_response::ListOffsetsResponse>, ApiRequestError> = client.request(
                    broker_address,
                    Request::of(listoffsets_request::ListOffsetsRequest { replica_id: -1, isolation_level: 0, topics: vec![topic] }),
                );
                listoffsets_response.and_then(|response| match response.response_message.responses.first() {
                    Some(topic_offsets) => Ok(topic_offsets.partition_responses.clone()),
                    None => Err(ApiRequestError::from("ListOffsets API did not return any topic offsets")),
                })
            })
            .collect::<Result<Vec<Vec<listoffsets_response::PartitionResponse>>, ApiRequestError>>()
            .map(|vecs| vecs.flatten());

        let partition_offsets = partition_offset_responses.map(|partition_offset_responses| {
            partition_offset_responses
                .into_iter()
                .map(|partition_response| (partition_response.partition, partition_response))
                .collect::<HashMap<i32, listoffsets_response::PartitionResponse>>()
        });

        partition_offsets.map(|offsets| (sorted_partition_metadata, offsets))
    }))
}

fn retrieve_consumer_offsets<T: ApiClientTrait + 'static>(
    client: IO<T, ApiRequestError>,
    group_id: &String,
    coordinator: &Coordinator,
    topic_metadata: &metadata_response::TopicMetadata,
    use_tls: bool,
) -> IO<HashMap<i32, offsetfetch_response::PartitionResponse>, ApiRequestError> {
    let group_id = group_id.clone();
    let coordinator = coordinator.clone();
    let topic_metadata = topic_metadata.clone();
    let coordinator_server = KafkaServerAddr::of(coordinator.host.clone(), coordinator.port, use_tls);

    client.and_then_result(Box::from(move |client: T| {
        let topic = offsetfetch_request::Topic {
            topic: topic_metadata.topic.clone(),
            partitions: topic_metadata.partition_metadata.iter().map(|p| p.partition).collect(),
        };

        let offsetfetch_result: Result<Response<offsetfetch_response::OffsetFetchResponse>, ApiRequestError> = client
            .request(
                &coordinator_server,
                Request::of(offsetfetch_request::OffsetFetchRequest { group_id: group_id.clone(), topics: vec![topic.clone()] }),
            )
            .and_then(|result: Response<offsetfetch_response::OffsetFetchResponse>| {
                if result.response_message.error_code != 0 {
                    Err(ApiRequestError::of(format!("Error code {} with OffsetFetchRequest", result.response_message.error_code)))
                } else {
                    Ok(result)
                }
            });

        let partition_responses = offsetfetch_result.and_then(|result| {
            let responses = result
                .response_message
                .responses
                .into_iter()
                .find(|response| response.topic.eq(&topic.topic))
                .map(|response| response.partition_responses);
            match responses {
                None => Err(ApiRequestError::from("Topic not returned from API request")),
                Some(partition_responses) => Ok(partition_responses),
            }
        });

        partition_responses.map(|partition_responses| {
            partition_responses.into_iter().map(|p| (p.partition, p)).collect::<HashMap<i32, offsetfetch_response::PartitionResponse>>()
        })
    }))
}

fn create_topic<T: ApiClientTrait + 'static>(
    client: IO<T, ApiRequestError>,
    topic: String,
    num_partitions: i32,
    replication_factor: i16,
    controller_broker: &metadata_response::BrokerMetadata,
    use_tls: bool,
    request_timeout_ms: i32,
) -> IO<Response<createtopics_response::CreateTopicsResponse>, ApiRequestError> {
    let controller_server = KafkaServerAddr::of(controller_broker.host.clone(), controller_broker.port, use_tls);

    client.and_then_result(Box::new(move |client: T| {
        client.request(
            &controller_server,
            Request::of(createtopics_request::CreateTopicsRequest {
                create_topic_requests: vec![createtopics_request::Request {
                    topic: topic.clone(),
                    num_partitions,
                    replication_factor,
                    replica_assignments: vec![],
                    config_entries: vec![],
                }],
                timeout: request_timeout_ms,
                validate_only: false,
            }),
        )
    }))
}

fn delete_topic<T: ApiClientTrait + 'static>(
    client: IO<T, ApiRequestError>,
    delete_topic_name: &String,
    controller_broker: &metadata_response::BrokerMetadata,
    use_tls: bool,
    request_timeout_ms: i32,
) -> IO<Response<deletetopics_response::DeleteTopicsResponse>, ApiRequestError> {
    let delete_topic_name = delete_topic_name.clone();
    let controller_broker = controller_broker.clone();
    let controller_server = KafkaServerAddr::of(controller_broker.host.clone(), controller_broker.port, use_tls);

    client.and_then_result(Box::new(move |client: T| {
        client.request(
            &controller_server,
            Request::of(deletetopics_request::DeleteTopicsRequest { topics: vec![delete_topic_name.clone()], timeout: request_timeout_ms }),
        )
    }))
}

fn alter_config<T: ApiClientTrait + 'static>(
    client: IO<T, ApiRequestError>,
    bootstrap_server: &KafkaServerAddr,
    resource: &alterconfigs_request::Resource,
) -> Result<(), StateFNError> {
    let resource = resource.clone();
    let bootstrap_server = bootstrap_server.clone();

    let alterconfigs_response: Result<Response<alterconfigs_response::AlterConfigsResponse>, ApiRequestError> = client
        .and_then_result(Box::new(move |client: T| {
            client.request(
                &bootstrap_server,
                Request::of(alterconfigs_request::AlterConfigsRequest { resources: vec![resource.clone()], validate_only: false }),
            )
        }))
        .into_result();

    alterconfigs_response.map_err(|tcp_error| StateFNError::caused("AlterConfigs request failed", tcp_error)).and_then(|alterconfigs_response| {
        match alterconfigs_response.response_message.resources.first() {
            None => Err(StateFNError::error("Missing resources from AlterConfigs request")),
            Some(resource) => {
                if resource.error_code == 0 {
                    Ok(())
                } else {
                    Err(StateFNError::Error(format!(
                        "AlterConfigs request failed with error code {}, {}",
                        resource.error_code,
                        resource.error_message.clone().unwrap_or(format!(""))
                    )))
                }
            }
        }
    })
}

#[cfg(test)]
#[path = "./event_bus_test.rs"]
mod event_bus_test;
