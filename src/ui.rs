use event_bus::State;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use kafka_protocol::protocol_responses::metadata_response::TopicMetadata;
use std::io::{stdin, stdout};
use termion;
use termion::color;
use termion::color::Color;
use termion::cursor;
use termion::raw::IntoRawMode;
use termion::style;
use std::thread;

pub fn update_with_state(state: &State) {
    if let Some(ref metadata) = state.metadata {
        clear_screen();
        show_topics(metadata, state.selected_index, &state.marked_deleted);
    }
}

pub fn clear_screen() {
    print!("{}{}{}", termion::clear::All, termion::cursor::Goto(1, 1), termion::cursor::Hide);
}

fn show_topics(metadata: &MetadataResponse, selected_index: usize, marked_deleted: &Vec<usize>) {

    let indexes = (0..metadata.topic_metadata.len());
    let indexed: Vec<(&TopicMetadata, usize)> = metadata.topic_metadata.iter().zip(indexes).collect();
    let delete_style = |i: &usize| marked_deleted.contains(i);

    indexed.iter().for_each(|&(topic, index)| {
        let line = index + 1;
        if selected_index == index {
            if delete_style(&index) {
                println!("{}{}{}{}{}{}", cursor::Goto(1, line as u16), color::Fg(color::Red), color::Bg(color::White), style::Underline, topic.topic, style::Reset);
            } else {
                println!("{}{}{}{}{}", cursor::Goto(1, line as u16), color::Fg(color::Red), color::Bg(color::White), topic.topic, style::Reset);
            }
        } else {
            if delete_style(&index) {
                println!("{}{}{}{}{}", cursor::Goto(1, line as u16), color::Fg(color::Cyan), style::Underline, topic.topic, style::Reset);
            } else {
                println!("{}{}{}{}", cursor::Goto(1, line as u16), color::Fg(color::Cyan), topic.topic, style::Reset);
            }
        }
    })

}