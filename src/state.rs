use kafka_protocol::protocol_responses::describeconfigs_response::Resource;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use kafka_protocol::protocol_responses::metadata_response::TopicMetadata;

#[derive(Clone)]
pub struct State {
    pub metadata: Option<MetadataResponse>,
    pub selected_index: usize,
    pub marked_deleted: Vec<String>,
    pub topic_info_state: Option<TopicInfoState>
}

impl State {

    pub fn new() -> State {
        State { metadata: None, selected_index: 0, marked_deleted: vec![], topic_info_state: None }
    }

    pub fn selected_topic_name(&self) -> Option<String> {
        self.metadata.iter().map(|metadata|{
            metadata.topic_metadata.get(self.selected_index).map(|m|{
                m.topic.clone()
            })
        }).collect::<Option<String>>()
    }

    pub fn selected_topic_metadata(&self) -> Option<TopicMetadata> {
        self.metadata.clone().and_then(|metadata|{
            metadata.topic_metadata.get(self.selected_index).map(|topic_metadata|{
                topic_metadata.clone()
            })
        })
    }
}

#[derive(Clone)]
pub struct TopicInfoState {
    pub topic_metadata: TopicMetadata,
    pub config_resource: Resource
}