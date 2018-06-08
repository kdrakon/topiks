use kafka_protocol::protocol_responses::describeconfigs_response::ConfigEntry;
use kafka_protocol::protocol_responses::describeconfigs_response::Resource;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use kafka_protocol::protocol_responses::metadata_response::TopicMetadata;
use state::State;
use state::TopicInfoState;
use std::io::{stdin, stdout, Write};
use std::io::Stdout;
use std::thread;
use termion;
use termion::color;
use termion::color::Color;
use termion::cursor;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;
use termion::style;
use termion::terminal_size;
use user_interface::topic_list::ListItem;
use user_interface::topic_list::ListItem::*;
use user_interface::topic_list::PagedVec;
use user_interface::topic_list::TopicList;
use utils;
use utils::pad_right;

pub fn update_with_state(state: &State) {
    let screen = &mut AlternateScreen::from(stdout().into_raw_mode().unwrap());
    let (width, height): (u16, u16) = terminal_size().unwrap();
    write!(screen, "{}", termion::clear::All).unwrap();

    if let Some(ref metadata) = state.metadata {
        if let Some(ref topic_info) = state.topic_info_state {
            show_topic_info(screen, topic_info, (width, height));
        } else {
            show_topics(screen, metadata, state.selected_index, &state.marked_deleted, (width, height));
        }
    }

    screen.flush().unwrap(); // flush complete buffer to screen once
}

fn show_topics(screen: &mut impl Write, metadata: &MetadataResponse, selected_index: usize, marked_deleted: &Vec<String>, (width, height): (u16, u16)) {
    let paged = PagedVec::from(&metadata.topic_metadata, (height - 1) as usize);

    if let Some((page_index, page)) = paged.page(selected_index) {
        let indexed = page.iter().zip((0..page.len())).collect::<Vec<(&&TopicMetadata, usize)>>();

        let list_items =
            indexed.iter().map(|&(topic_metadata, index)| {
                let topic_name = topic_metadata.topic.clone();
                if page_index == index {
                    Selected(topic_name)
                } else if marked_deleted.contains(&topic_name) {
                    Deleted(topic_name)
                } else {
                    Normal(topic_name)
                }
            }).collect::<Vec<ListItem>>();

        (TopicList { list: list_items }).display(screen, (1, 1));
    }
}

fn show_topic_info(screen: &mut impl Write, topic_info: &TopicInfoState, (width, height): (u16, u16)) {
    let ref topic_metadata = topic_info.topic_metadata;
    let ref config_resource = topic_info.config_resource;

    // header
    write!(screen, "{}{}{}", style::Bold, pad_right(&format!("Topic: {}", &topic_metadata.topic), width), style::Reset).unwrap();
    write!(screen, "{}", pad_right(&format!("Internal: {}", &(utils::bool_yes_no(topic_metadata.is_internal))), width)).unwrap();

    // partitions
    write!(screen, "{}{}{}", style::Bold, pad_right(&String::from("Partitions"), width), style::Reset).unwrap();
    topic_metadata.partition_metadata.iter().for_each(|partition| {
        let partition_header = &format!("Partition#: {:3} ", partition.partition);
        let partition_details = &format!("Leader: {} Replicas: {:?} Offline Replicas: {:?} ISR: {:?}", partition.leader, partition.replicas, partition.offline_replicas, partition.isr);
        write!(screen, "{}{}{}", color::Fg(color::Cyan), partition_header, style::Reset).unwrap();
        write!(screen, "{}", pad_right(partition_details, width - partition_header.len() as u16)).unwrap();
    });

    // configs
    write!(screen, "{}{}{}", style::Bold, pad_right(&String::from("Configs:"), width), style::Reset).unwrap();
    let longest_config_name_len = config_resource.config_entries.iter().map(|config_entry| config_entry.config_name.len()).max().unwrap();
    config_resource.config_entries.iter().cloned().for_each(|config_entry| {
        let config_name = pad_right(&config_entry.config_name, (longest_config_name_len as u16) + 1);
        write!(screen, "{}", pad_right(&format!("{}: {}", config_name, config_entry.config_value.unwrap_or(String::from("n/a"))), width)).unwrap();
    });
}