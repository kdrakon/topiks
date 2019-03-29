use std::collections::HashMap;
use std::fmt::Display;

use crate::kafka_protocol::protocol_responses::describeconfigs_response::Resource;
use crate::kafka_protocol::protocol_responses::listoffsets_response;
use crate::kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use crate::kafka_protocol::protocol_responses::metadata_response::PartitionMetadata;
use crate::kafka_protocol::protocol_responses::metadata_response::TopicMetadata;
use crate::kafka_protocol::protocol_responses::offsetfetch_response;
use crate::state::CurrentView::*;

#[derive(Clone)]
pub struct State {
    pub dialog_message: Option<DialogMessage>,
    pub user_input: Option<String>,
    pub current_view: CurrentView,
    pub metadata: Option<MetadataResponse>,
    pub selected_index: usize,
    pub marked_deleted: Vec<String>,
    pub topic_name_query: Option<String>,
    pub topic_info_state: Option<TopicInfoState>,
    pub partition_info_state: Option<PartitionInfoState>,
}

#[derive(Clone)]
pub enum DialogMessage {
    None,
    Warn(String),
    Info(String),
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CurrentView {
    Topics,
    Partitions,
    TopicInfo,
    HelpScreen,
}

pub type StateFn<T> = Box<Fn(&State) -> Result<T, StateFNError>>;

pub enum StateFNError {
    Error(String),
    Caused(String, Box<Display>),
}

impl StateFNError {
    pub fn caused<T: Display + 'static>(error: &str, cause: T) -> StateFNError {
        StateFNError::Caused(String::from(error), Box::from(cause))
    }
    pub fn error(error: &str) -> StateFNError {
        StateFNError::Error(String::from(error))
    }
}

impl State {
    pub fn new() -> State {
        State {
            dialog_message: None,
            user_input: None,
            current_view: Topics,
            metadata: None,
            selected_index: 0,
            marked_deleted: vec![],
            topic_name_query: None,
            topic_info_state: None,
            partition_info_state: None,
        }
    }

    pub fn selected_topic_name(&self) -> Option<String> {
        self.metadata.as_ref().and_then(|metadata| metadata.topic_metadata.get(self.selected_index).map(|m: &TopicMetadata| m.topic.clone()))
    }

    pub fn selected_topic_metadata(&self) -> Option<TopicMetadata> {
        self.metadata.as_ref().and_then(|metadata| metadata.topic_metadata.get(self.selected_index).map(|topic_metadata| topic_metadata.clone()))
    }

    pub fn find_next_index(&self, in_reverse: bool) -> Option<usize> {
        self.topic_name_query.as_ref().and_then(|query| {
            self.metadata.as_ref().and_then(|metadata| {
                let indexed = metadata.topic_metadata.iter().zip(0..metadata.topic_metadata.len()).collect::<Vec<(&TopicMetadata, usize)>>();
                let slice = if self.selected_index < metadata.topic_metadata.len() {
                    if in_reverse {
                        let mut vec = indexed.as_slice()[0..self.selected_index].to_vec();
                        vec.reverse();
                        vec
                    } else {
                        indexed.as_slice()[(self.selected_index + 1)..].to_vec() // +1 since we don't want to find the selected index
                    }
                } else {
                    vec![]
                };

                slice
                    .iter()
                    .find(|m| {
                        let (topic_metadata, _index) = **m;
                        topic_metadata.topic.contains(query)
                    })
                    .map(|result| result.1)
            })
        })
    }
}

#[derive(Clone)]
pub struct TopicInfoState {
    pub topic_metadata: TopicMetadata,
    pub config_resource: Resource,
    pub selected_index: usize,
    pub configs_marked_deleted: Vec<String>,
    pub configs_marked_modified: Vec<String>,
}

impl TopicInfoState {
    pub fn new(topic_metadata: TopicMetadata, config_resource: Resource) -> TopicInfoState {
        TopicInfoState { topic_metadata, config_resource, selected_index: 0, configs_marked_deleted: vec![], configs_marked_modified: vec![] }
    }
}

#[derive(Clone)]
pub struct PartitionInfoState {
    pub selected_index: usize,
    pub partition_metadata: Vec<PartitionMetadata>,
    pub partition_offsets: HashMap<i32, listoffsets_response::PartitionResponse>,
    pub consumer_offsets: HashMap<i32, offsetfetch_response::PartitionResponse>,
}

impl PartitionInfoState {
    pub fn new(
        partition_metadata: Vec<PartitionMetadata>,
        partition_offsets: HashMap<i32, listoffsets_response::PartitionResponse>,
        consumer_offsets: HashMap<i32, offsetfetch_response::PartitionResponse>,
    ) -> PartitionInfoState {
        PartitionInfoState { selected_index: 0, partition_metadata, partition_offsets, consumer_offsets }
    }
}
