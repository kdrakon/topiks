use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use kafka_protocol::protocol_responses::metadata_response::TopicMetadata;
use state::State;
use std::io::{stdin, stdout, Write};
use std::thread;
use termion;
use termion::color;
use termion::color::Color;
use termion::cursor;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::style;
use termion::terminal_size;
use utils;
use utils::pad_right;
use std::io::Stdout;
use termion::raw::RawTerminal;

pub fn update_with_state(state: &State) {

    let screen = &mut AlternateScreen::from(stdout());

    let (width, height): (u16, u16) = terminal_size().unwrap();

    if let Some(ref metadata) = state.metadata {
        if state.show_selected_topic_info {
            show_topic_info(screen, metadata.topic_metadata.get(state.selected_index), (width, height));
        } else {
            show_topics(screen, metadata, state.selected_index, &state.marked_deleted, (width, height));
        }
    }

    screen.flush().unwrap(); // flush complete buffer to screen once
}

fn show_topics(screen: &mut AlternateScreen<Stdout>, metadata: &MetadataResponse, selected_index: usize, marked_deleted: &Vec<usize>, (width, height): (u16, u16)) {

    let indexes = (0..metadata.topic_metadata.len());
    let indexed: Vec<(&TopicMetadata, usize)> = metadata.topic_metadata.iter().zip(indexes).collect();
    let delete_style = |i: &usize| marked_deleted.contains(i);

    indexed.iter().for_each(|&(topic, index)| {

        let topic_label = pad_right(&topic.topic, width);
        let line = index + 1;
        if selected_index == index {
            if delete_style(&index) {
                write!(screen, "{}{}{}{}{}", color::Fg(color::Black), color::Bg(color::White), style::Underline, topic_label, style::Reset);
            } else {
                write!(screen, "{}{}{}{}", color::Fg(color::Black), color::Bg(color::White), topic_label, style::Reset);
            }
        } else {
            if delete_style(&index) {
                write!(screen, "{}{}{}{}", color::Fg(color::Green), color::Bg(color::Red), topic_label, style::Reset);
            } else {
                write!(screen, "{}{}{}", color::Fg(color::Cyan), topic_label, style::Reset);
            }
        }
    });
}

fn show_topic_info(screen: &mut AlternateScreen<Stdout>, topic_metadata: Option<&TopicMetadata>, (width, height): (u16, u16)) {

    let display = |screen: &mut AlternateScreen<Stdout>, topic_metadata: &TopicMetadata| -> () {
        write!(screen, "{}{}{}", style::Bold, pad_right(&format!("Topic: {}", &topic_metadata.topic), width), style::Reset);
        write!(screen, "{}", pad_right(&format!("Internal: {}", &(utils::bool_yes_no(topic_metadata.is_internal))), width));

        write!(screen, "{}{}{}", style::Bold, pad_right(&String::from("Partitions"), width), style::Reset);
        topic_metadata.partition_metadata.iter().for_each(|partition|{
            let partition_header = &format!("Partition#: {:3} ", partition.partition);
            let partition_details = &format!("Leader: {} Replicas: {:?} Offline Replicas: {:?} ISR: {:?}", partition.leader, partition.replicas, partition.offline_replicas, partition.isr);
            write!(screen, "{}{}{}", color::Fg(color::Cyan), partition_header, style::Reset);
            write!(screen, "{}", pad_right(partition_details, width - partition_header.len() as u16));
        });
    };

    match topic_metadata {
        Some(topic_metadata) => display(screen, topic_metadata),
        None => {
            writeln!(screen, "Error: could not find topic metadata");
        }
    };
}