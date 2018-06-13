use kafka_protocol::protocol_responses::describeconfigs_response::Resource;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use kafka_protocol::protocol_responses::metadata_response::TopicMetadata;
use state::CurrentView::*;

#[derive(Clone)]
pub struct State {
    pub current_view: CurrentView,
    pub metadata: Option<MetadataResponse>,
    pub selected_index: usize,
    pub marked_deleted: Vec<String>,
    pub topic_name_query: Option<String>,
    pub topic_info_state: Option<TopicInfoState>,
    pub partition_info_state: Option<PartitionInfoState>
}

#[derive(Clone)]
pub enum CurrentView {
    Topics,
    Partitions,
    TopicInfo,
}

impl State {
    pub fn new() -> State {
        State { current_view: Topics, metadata: None, selected_index: 0, marked_deleted: vec![], topic_name_query: None, topic_info_state: None, partition_info_state: None }
    }

    pub fn selected_topic_name(&self) -> Option<String> {
        self.metadata.as_ref().and_then(|metadata| {
            metadata.topic_metadata.get(self.selected_index).map(|m: &TopicMetadata| {
                m.topic.clone()
            })
        })
    }

    pub fn selected_topic_metadata(&self) -> Option<TopicMetadata> {
        self.metadata.as_ref().and_then(|metadata| {
            metadata.topic_metadata.get(self.selected_index).map(|topic_metadata| {
                topic_metadata.clone()
            })
        })
    }

    pub fn find_next_index(&self, in_reverse: bool) -> Option<usize> {
        self.topic_name_query.as_ref().and_then(|query| {
            self.metadata.as_ref().and_then(|metadata| {
                let indexed = metadata.topic_metadata.iter().zip((0..metadata.topic_metadata.len())).collect::<Vec<(&TopicMetadata, usize)>>();
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

                slice.iter().find(|m| {
                    let (topic_metadata, index) = **m;
                    topic_metadata.topic.contains(query)
                }).map(|result| result.1)
            })
        })
    }
}

#[derive(Clone)]
pub struct TopicInfoState {
    pub topic_metadata: TopicMetadata,
    pub config_resource: Resource,
}

#[derive(Clone)]
pub struct PartitionInfoState {
    pub selected_index: usize
}