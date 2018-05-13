use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;

#[derive(Clone)]
pub struct State {
    pub metadata: Option<MetadataResponse>,
    pub selected_index: usize,
    pub show_selected_topic_info: bool,
    pub marked_deleted: Vec<usize>
}

impl State {
    pub fn new() -> State {
        State { metadata: None, selected_index: 0, marked_deleted: vec![], show_selected_topic_info: false }
    }
    pub fn with_metadata(&self, metadata: MetadataResponse) -> State {
        State { metadata: Some(metadata.clone()), selected_index: self.selected_index, marked_deleted: self.marked_deleted.clone(), show_selected_topic_info: self.show_selected_topic_info }
    }
    pub fn with_selected_index(&self, selected_index: usize) -> State {
        State { metadata: self.metadata.clone(), selected_index, marked_deleted: self.marked_deleted.clone(), show_selected_topic_info: self.show_selected_topic_info }
    }
    pub fn with_marked_deleted(&self, marked_deleted: Vec<usize>) -> State {
        State { metadata: self.metadata.clone(), selected_index: self.selected_index, marked_deleted, show_selected_topic_info: self.show_selected_topic_info }
    }
    pub fn with_show_selected_topic_info(&self, show_selected_topic_info: bool) -> State {
        State { metadata: self.metadata.clone(), selected_index: self.selected_index, marked_deleted: self.marked_deleted.clone(), show_selected_topic_info }
    }
}