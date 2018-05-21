use kafka_protocol::protocol_responses::describeconfigs_response::Resource;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;

#[derive(Clone)]
pub struct State {
    pub metadata: Option<MetadataResponse>,
    pub selected_index: usize,
    pub marked_deleted: Vec<usize>,
    pub show_selected_topic_info: bool,
    pub topic_info_state: Option<TopicInfoState>
}

impl State {
    pub fn new() -> State {
        State { metadata: None, selected_index: 0, marked_deleted: vec![], show_selected_topic_info: false, topic_info_state: None }
    }
}

#[derive(Clone)]
pub struct TopicInfoState {
    pub config_info: Resource
}