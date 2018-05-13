use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use kafka_protocol::protocol_responses::metadata_response::TopicMetadata;
use state::State;
use std::io::{stdin, stdout};
use std::thread;
use termion;
use termion::color;
use termion::color::Color;
use termion::cursor;
use termion::raw::IntoRawMode;
use termion::style;
use termion::terminal_size;
use utils;

pub fn update_with_state(state: &State) {

    let (width, height): (u16, u16) = terminal_size().unwrap();

    if let Some(ref metadata) = state.metadata {
        clear_screen();
        if state.show_selected_topic_info {
            show_topic_info();
        } else {
            show_topics(metadata, state.selected_index, &state.marked_deleted, (width, height));
        }
    }
}

pub fn clear_screen() {
    print!("{}{}{}", termion::clear::All, termion::cursor::Goto(1, 1), termion::cursor::Hide);
}

fn show_topics(metadata: &MetadataResponse, selected_index: usize, marked_deleted: &Vec<usize>, (width, height): (u16, u16)) {

    let indexes = (0..metadata.topic_metadata.len());
    let indexed: Vec<(&TopicMetadata, usize)> = metadata.topic_metadata.iter().zip(indexes).collect();
    let delete_style = |i: &usize| marked_deleted.contains(i);

    indexed.iter().for_each(|&(topic, index)| {
        let topic_label = utils::pad_right(&topic.topic, width);
        let line = index + 1;
        if selected_index == index {
            if delete_style(&index) {
                println!("{}{}{}{}{}{}", cursor::Goto(1, line as u16), color::Fg(color::Black), color::Bg(color::White), style::Underline, topic_label, style::Reset);
            } else {
                println!("{}{}{}{}{}", cursor::Goto(1, line as u16), color::Fg(color::Black), color::Bg(color::White), topic_label, style::Reset);
            }
        } else {
            if delete_style(&index) {
                println!("{}{}{}{}{}", cursor::Goto(1, line as u16), color::Fg(color::Red), style::Underline, topic_label, style::Reset);
            } else {
                println!("{}{}{}{}", cursor::Goto(1, line as u16), color::Fg(color::Cyan), topic_label, style::Reset);
            }
        }
    })
}

fn show_topic_info() {
    println!("info goes here");
}