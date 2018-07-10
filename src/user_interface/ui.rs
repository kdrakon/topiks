use kafka_protocol::protocol_responses::describeconfigs_response::ConfigEntry;
use kafka_protocol::protocol_responses::describeconfigs_response::Resource;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use kafka_protocol::protocol_responses::metadata_response::PartitionMetadata;
use kafka_protocol::protocol_responses::metadata_response::TopicMetadata;
use state::CurrentView;
use state::PartitionInfoState;
use state::State;
use state::TopicInfoState;
use state::UIMessage;
use std::io::{stdin, stdout, Write};
use std::io::Stdout;
use std::thread;
use termion;
use termion::clear;
use termion::color;
use termion::color::Color;
use termion::cursor;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;
use termion::style;
use termion::terminal_size;
use user_interface::selectable_list::PartitionListItem;
use user_interface::selectable_list::SelectableList;
use user_interface::selectable_list::TopicListItem;
use util::paged_vec::PagedVec;
use util::utils;
use util::utils::pad_right;
use user_interface::selectable_list::TopicConfigurationItem;

pub fn update_with_state(state: &State) {
    let screen = &mut AlternateScreen::from(stdout().into_raw_mode().unwrap());
    let (width, height): (u16, u16) = terminal_size().unwrap();
    write!(screen, "{}", termion::clear::All).unwrap();

    show_message(screen, width, &state.message);

    if let Some(ref metadata) = state.metadata {
        match state.current_view {
            CurrentView::Topics => {
                show_topics(screen, (width, height - 2), (1, 2), metadata, state.selected_index, &state.marked_deleted);
            }
            CurrentView::Partitions => {
                if let Some(partition_info_state) = state.partition_info_state.as_ref() {
                    show_topic_partitions(screen, (width, height - 2), (1, 2), partition_info_state);
                }
            }
            CurrentView::TopicInfo => {
                if let Some(ref topic_info) = state.topic_info_state {
                    show_topic_info(screen, (width, height - 4), (1, 4), topic_info);
                }
            }
        }
    }

    show_user_input(screen, (width, height), state.user_input.as_ref());

    screen.flush().unwrap(); // flush complete buffer to screen once
}

fn show_message(screen: &mut impl Write, width: u16, message: &Option<UIMessage>) {
    write!(screen, "{}{}", cursor::Goto(1, 1), color::Fg(color::Black)).unwrap();
    match message.as_ref() {
        None => (),
        Some(&UIMessage::None) => (),
        Some(UIMessage::Error(error)) => write!(screen, "{}{}", color::Bg(color::LightRed), pad_right(&error, width)).unwrap(),
        Some(UIMessage::Warn(warn)) => write!(screen, "{}{}", color::Bg(color::LightYellow), pad_right(&warn, width)).unwrap(),
        Some(UIMessage::Info(info)) => write!(screen, "{}{}", color::Bg(color::LightBlue), pad_right(&info, width)).unwrap()
    }
    write!(screen, "{}", style::Reset).unwrap();
}

fn show_topics(screen: &mut impl Write, (width, height): (u16, u16), (start_x, start_y): (u16, u16), metadata: &MetadataResponse, selected_index: usize, marked_deleted: &Vec<String>) {
    use user_interface::selectable_list::TopicListItem::*;

    let paged = PagedVec::from(&metadata.topic_metadata, height as usize);

    if let Some((page_index, page)) = paged.page(selected_index) {
        let indexed = page.iter().zip((0..page.len())).collect::<Vec<(&&TopicMetadata, usize)>>();
        let list_items =
            indexed.iter().map(|&(topic_metadata, index)| {
                let topic_name = topic_metadata.topic.clone();
                let partitions = topic_metadata.partition_metadata.len();

                let item =
                    if marked_deleted.contains(&topic_name) {
                        Deleted(topic_name, partitions)
                    } else if topic_metadata.is_internal {
                        Internal(topic_name, partitions)
                    } else {
                        Normal(topic_name, partitions)
                    };
                if page_index == index { Selected(Box::from(item)) } else { item }
            }).collect::<Vec<TopicListItem>>();

        (SelectableList { list: list_items }).display(screen, (start_x, start_y), width);
    }
}

fn show_topic_partitions(screen: &mut impl Write, (width, height): (u16, u16), (start_x, start_y): (u16, u16), partition_info_state: &PartitionInfoState) {
    use user_interface::selectable_list::PartitionListItem::*;

    let paged = PagedVec::from(&partition_info_state.partition_metadata, height as usize);

    if let Some((page_index, page)) = paged.page(partition_info_state.selected_index) {
        let indexed = page.iter().zip((0..page.len())).collect::<Vec<(&&PartitionMetadata, usize)>>();
        let list_items =
            indexed.iter().map(|&(partition_metadata, index)| {
                let consumer_offset = partition_info_state.consumer_offsets.get(&partition_metadata.partition).map(|p| p.offset).unwrap_or(-1);
                let partition_offset = partition_info_state.partition_offsets.get(&partition_metadata.partition).map(|p| p.offset).unwrap_or(-1);

                let item = Normal { partition: partition_metadata.partition, partition_metadata: (*partition_metadata).clone(), consumer_offset, partition_offset };
                if page_index == index { Selected(Box::from(item)) } else { item }
            }).collect::<Vec<PartitionListItem>>();

        (SelectableList { list: list_items }).display(screen, (start_x, start_y), width);
    }
}

fn show_topic_info(screen: &mut impl Write, (width, height): (u16, u16), (start_x, start_y): (u16, u16), topic_info: &TopicInfoState) {
    use user_interface::selectable_list::TopicConfigurationItem::*;

    let ref topic_metadata = topic_info.topic_metadata;
    let ref config_resource = topic_info.config_resource;

    // header
    write!(screen, "{}{}{}", style::Bold, pad_right(&format!("Topic: {}", &topic_metadata.topic), width), style::Reset).unwrap();
    write!(screen, "{}", pad_right(&format!("Internal: {}", &(utils::bool_yes_no(topic_metadata.is_internal))), width)).unwrap();

    // configs
    write!(screen, "{}{}{}", style::Bold, pad_right(&String::from("Configs:"), width), style::Reset).unwrap();
    let longest_config_name_len = config_resource.config_entries.iter().map(|config_entry| config_entry.config_name.len()).max().unwrap() as u16;
    let paged = PagedVec::from(&config_resource.config_entries, height as usize);

    if let Some((page_index, page)) = paged.page(topic_info.selected_index) {
        let indexed = page.iter().zip((0..page.len())).collect::<Vec<(&&ConfigEntry, usize)>>();
        let list_items =
            indexed.iter().map(|&(config_entry, index)| {
                let item = Config { name: pad_right(&config_entry.config_name, longest_config_name_len), value: config_entry.config_value.clone() };
                let item = if page_index == index { Selected(Box::from(item)) } else { item };
                let item = if config_entry.read_only { ReadOnlyConfig(Box::from(item)) } else { item };
                let item = if config_entry.is_sensitive { SensitiveConfig(Box::from(item)) } else { item };
                item
            }).collect::<Vec<TopicConfigurationItem>>();

        (SelectableList { list: list_items }).display(screen, (start_x, start_y), width);
    }
}

fn show_user_input(screen: &mut impl Write, (width, height): (u16, u16), user_input: Option<&String>) {
    write!(screen, "{}", cursor::Goto(1, height)).unwrap();
    match user_input {
        None => write!(screen, "{}", clear::CurrentLine).unwrap(),
        Some(input) => write!(screen, "{}{}", clear::CurrentLine, input).unwrap()
    }
}