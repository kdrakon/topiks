use kafka_protocol::protocol_responses::describeconfigs_response::Resource;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;

#[derive(Clone)]
pub struct State {
    pub metadata: Option<MetadataResponse>,
    pub selected_index: usize,
    pub marked_deleted: Vec<usize>,
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
}

#[derive(Clone)]
pub struct TopicInfoState {
    pub config_info: Vec<Resource>
}