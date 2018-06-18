use kafka_protocol::protocol_responses::describeconfigs_response::ConfigEntry;
use kafka_protocol::protocol_responses::describeconfigs_response::Resource;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use kafka_protocol::protocol_responses::metadata_response::TopicMetadata;
use state::CurrentView;
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
use termion::clear;
use termion::terminal_size;
use user_interface::selectable_list::ListItem;
use user_interface::selectable_list::ListItem::*;
use user_interface::selectable_list::SelectableList;
use utils;
use utils::pad_right;
use utils::PagedVec;

pub fn update_with_state(state: &State) {
    let screen = &mut AlternateScreen::from(stdout().into_raw_mode().unwrap());
    let (width, height): (u16, u16) = terminal_size().unwrap();
    write!(screen, "{}", termion::clear::All).unwrap();

    if let Some(ref metadata) = state.metadata {
        match state.current_view {
            CurrentView::Topics => {
                show_topics(screen, (width, height), metadata, state.selected_index, &state.marked_deleted);
            }
            CurrentView::Partitions => {
                if let Some(topic_metadata) = metadata.topic_metadata.get(state.selected_index).as_ref() {
                    show_topic_partitions(screen, (width, height), topic_metadata);
                }
            }
            CurrentView::TopicInfo => {
                if let Some(ref topic_info) = state.topic_info_state {
                    show_topic_info(screen, (width, height), topic_info);
                }
            }
        }
    }

    show_user_input(screen, (width, height), state.user_input.as_ref());

    screen.flush().unwrap(); // flush complete buffer to screen once
}

fn show_topics(screen: &mut impl Write, (width, height): (u16, u16), metadata: &MetadataResponse, selected_index: usize, marked_deleted: &Vec<String>) {
    let paged = PagedVec::from(&metadata.topic_metadata, (height - 1) as usize);

    if let Some((page_index, page)) = paged.page(selected_index) {
        let indexed = page.iter().zip((0..page.len())).collect::<Vec<(&&TopicMetadata, usize)>>();
        let list_items =
            indexed.iter().map(|&(topic_metadata, index)| {
                let topic_name = topic_metadata.topic.clone();
                let topic_label = format!("{} [{}]", topic_name, topic_metadata.partition_metadata.len());
                if page_index == index {
                    Selected(topic_label)
                } else if marked_deleted.contains(&topic_name) {
                    Deleted(topic_name)
                } else {
                    Normal(topic_label)
                }
            }).collect::<Vec<ListItem>>();

        (SelectableList { list: list_items }).display(screen, (1, 1));
    }
}

fn show_topic_partitions(screen: &mut impl Write, (width, height): (u16, u16), topic_metadata: &TopicMetadata) {
    // TODO page this
    let partition_metadata = &topic_metadata.partition_metadata;
//    let paged = PagedVec::from(partition_metadata, (height - 1) as usize);

    let list_items =
        partition_metadata.iter().map(|partition| {
            Normal(format!(
                "{}Partition#{:3}{} -- Leader: {:4} Replicas: {:?} Offline Replicas: {:?} ISR: {:?}",
                style::Bold, partition.partition, style::Reset, partition.leader, partition.replicas, partition.offline_replicas, partition.isr
            ))
        }).collect::<Vec<ListItem>>();

    (SelectableList { list: list_items }).display(screen, (1, 1));
}

fn show_topic_info(screen: &mut impl Write, (width, height): (u16, u16), topic_info: &TopicInfoState) {
    let ref topic_metadata = topic_info.topic_metadata;
    let ref config_resource = topic_info.config_resource;

    // header
    write!(screen, "{}{}{}", style::Bold, pad_right(&format!("Topic: {}", &topic_metadata.topic), width), style::Reset).unwrap();
    write!(screen, "{}", pad_right(&format!("Internal: {}", &(utils::bool_yes_no(topic_metadata.is_internal))), width)).unwrap();

    // configs
    write!(screen, "{}{}{}", style::Bold, pad_right(&String::from("Configs:"), width), style::Reset).unwrap();
    let longest_config_name_len = config_resource.config_entries.iter().map(|config_entry| config_entry.config_name.len()).max().unwrap();
    config_resource.config_entries.iter().cloned().for_each(|config_entry| {
        let config_name = pad_right(&config_entry.config_name, (longest_config_name_len as u16) + 1);
        write!(screen, "{}", pad_right(&format!("{}: {}", config_name, config_entry.config_value.unwrap_or(String::from("n/a"))), width)).unwrap();
    });
}

fn show_user_input(screen: &mut impl Write, (width, height): (u16, u16), user_input: Option<&String>) {
    write!(screen, "{}", cursor::Goto(1, height)).unwrap();
    match user_input {
        None => write!(screen, "{}", clear::CurrentLine).unwrap(),
        Some(input) => write!(screen, "{}{}", clear::CurrentLine, input).unwrap()
    }
}