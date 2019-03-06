use std::io::{stdout, Write};

use termion::clear;
use termion::color;
use termion::cursor;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::style;
use termion::terminal_size;

use kafka_protocol::protocol_responses::describeconfigs_response::ConfigEntry;
use kafka_protocol::protocol_responses::describeconfigs_response::ConfigSource;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use kafka_protocol::protocol_responses::metadata_response::PartitionMetadata;
use kafka_protocol::protocol_responses::metadata_response::TopicMetadata;
use state::CurrentView;
use state::DialogMessage;
use state::PartitionInfoState;
use state::State;
use state::TopicInfoState;
use user_interface::selectable_list::PartitionListItem;
use user_interface::selectable_list::SelectableList;
use user_interface::selectable_list::TopicConfigurationItem;
use user_interface::selectable_list::TopicListItem;
use util::paged_vec::PagedVec;
use util::utils;
use util::utils::pad_right;

pub fn update_with_state(state: &State) {
    let screen = &mut AlternateScreen::from(stdout().into_raw_mode().unwrap());
    let (width, height): (u16, u16) = terminal_size().unwrap();

    if let Some(ref metadata) = state.metadata {

        show_dialog_header(screen, width, metadata, &state.dialog_message);

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
                    show_topic_info(screen, (width, height - 4), (1, 2), topic_info);
                }
            }
            CurrentView::HelpScreen => show_help(screen),
        }
    }

    show_user_input(screen, (width, height), state.user_input.as_ref());

    screen.flush().unwrap(); // flush complete buffer to screen once
}

const HELP: [(&str, &str); 15] = [
    ("h", "Toggle this screen"),
    ("q", "Quit"),
    ("p", "Toggle partitions"),
    ("i", "Toggle topic config"),
    ("/", "Enter search query for topic name"),
    ("n", "Find next search result"),
    ("r", "Refresh. Retrieves metadata from Kafka cluster"),
    (":", "Modify a resource (e.g. topic config) via text input"),
    ("d", "Delete a resource. Will delete a topic or reset a topic config"),
    ("Up⬆", "Move up one topic"),
    ("Down⬇", "Move down one topic"),
    ("PgUp⇞", "Move up ten topics"),
    ("PgDown⇟", "Move down ten topics"),
    ("Home⤒", "Go to first topic"),
    ("End⤓", "Go to last topic"),
];

fn show_help(screen: &mut impl Write) {
    for (index, (key, help)) in HELP.iter().enumerate() {
        write!(screen, "\n{}{}{}{} → {}", cursor::Goto(2, (index + 2) as u16), style::Bold, key, style::Reset, help).unwrap();
    }
}

fn show_dialog_header(screen: &mut impl Write, width: u16, metadata: &MetadataResponse, message: &Option<DialogMessage>) {
    write!(screen, "{}{}", cursor::Goto(1, 1), color::Fg(color::Black)).unwrap();
    match message.as_ref() {
        None => {
            let cluster_name = metadata.cluster_id.as_ref().map(|s| s.as_str()).unwrap_or("unknown");
            let header = format!("cluster:{} brokers:{} topics:{}", cluster_name, metadata.brokers.len(), metadata.topic_metadata.len());
            write!(screen, "{}{}{}{}", color::Fg(color::White), cursor::Right(width - (header.len() as u16)), style::Bold, header).unwrap();
        }
        Some(&DialogMessage::None) => (),
        Some(DialogMessage::Error(error)) => write!(screen, "{}{}", color::Bg(color::LightRed), pad_right(&error, width)).unwrap(),
        Some(DialogMessage::Warn(warn)) => write!(screen, "{}{}", color::Bg(color::LightYellow), pad_right(&warn, width)).unwrap(),
        Some(DialogMessage::Info(info)) => write!(screen, "{}{}", color::Bg(color::LightBlue), pad_right(&info, width)).unwrap(),
    }
    write!(screen, "{}", style::Reset).unwrap();
}

fn show_topics(
    screen: &mut impl Write,
    (width, height): (u16, u16),
    (start_x, start_y): (u16, u16),
    metadata: &MetadataResponse,
    selected_index: usize,
    marked_deleted: &Vec<String>,
) {
    use user_interface::selectable_list::TopicListItem::*;

    let paged = PagedVec::from(&metadata.topic_metadata, height as usize);

    if let Some((page_index, page)) = paged.page(selected_index) {
        let indexed = page.iter().zip(0..page.len()).collect::<Vec<(&&TopicMetadata, usize)>>();
        let list_items = indexed
            .iter()
            .map(|&(topic_metadata, index)| {
                let topic_name = topic_metadata.topic.clone();
                let partitions = topic_metadata.partition_metadata.len();

                let item = if marked_deleted.contains(&topic_name) {
                    Deleted(topic_name, partitions)
                } else if topic_metadata.is_internal {
                    Internal(topic_name, partitions)
                } else {
                    Normal(topic_name, partitions)
                };
                if page_index == index {
                    Selected(Box::from(item))
                } else {
                    item
                }
            })
            .collect::<Vec<TopicListItem>>();

        (SelectableList { list: list_items }).display(screen, (start_x, start_y), width);
    }
}

fn show_topic_partitions(
    screen: &mut impl Write,
    (width, height): (u16, u16),
    (start_x, start_y): (u16, u16),
    partition_info_state: &PartitionInfoState,
) {
    use user_interface::selectable_list::PartitionListItem::*;

    let paged = PagedVec::from(&partition_info_state.partition_metadata, height as usize);

    if let Some((page_index, page)) = paged.page(partition_info_state.selected_index) {
        let indexed = page.iter().zip(0..page.len()).collect::<Vec<(&&PartitionMetadata, usize)>>();
        let list_items = indexed
            .iter()
            .map(|&(partition_metadata, index)| {
                let consumer_offset = partition_info_state.consumer_offsets.get(&partition_metadata.partition).map(|p| p.offset).unwrap_or(-1);
                let partition_offset = partition_info_state.partition_offsets.get(&partition_metadata.partition).map(|p| p.offset).unwrap_or(-1);

                let item = Normal {
                    partition: partition_metadata.partition,
                    partition_metadata: (*partition_metadata).clone(),
                    consumer_offset,
                    partition_offset,
                };
                if page_index == index {
                    Selected(Box::from(item))
                } else {
                    item
                }
            })
            .collect::<Vec<PartitionListItem>>();

        (SelectableList { list: list_items }).display(screen, (start_x, start_y), width);
    }
}

fn show_topic_info(screen: &mut impl Write, (width, height): (u16, u16), (start_x, start_y): (u16, u16), topic_info: &TopicInfoState) {
    use user_interface::selectable_list::TopicConfigurationItem::*;

    let ref topic_metadata = topic_info.topic_metadata;
    let ref config_resource = topic_info.config_resource;

    // header
    write!(
        screen,
        "{}{}{}{}",
        cursor::Goto(start_x, start_y),
        style::Bold,
        pad_right(&format!("Topic: {}", &topic_metadata.topic), width),
        style::Reset
    )
    .unwrap();
    write!(screen, "{}", pad_right(&format!("Internal: {}", &(utils::bool_yes_no(topic_metadata.is_internal))), width)).unwrap();

    // configs
    write!(screen, "{}{}{}", style::Bold, pad_right(&String::from("Configs:"), width), style::Reset).unwrap();
    let longest_config_name_len = config_resource.config_entries.iter().map(|config_entry| config_entry.config_name.len()).max().unwrap_or(0) as u16;
    let paged = PagedVec::from(&config_resource.config_entries, (height - 1) as usize);

    if let Some((page_index, page)) = paged.page(topic_info.selected_index) {
        let indexed = page.iter().zip(0..page.len()).collect::<Vec<(&&ConfigEntry, usize)>>();
        let list_items = indexed
            .iter()
            .map(|&(config_entry, index)| {
                let item = Config { name: pad_right(&config_entry.config_name, longest_config_name_len), value: config_entry.config_value.clone() };
                let item = if page_index == index { Selected(Box::from(item)) } else { item };
                let item = if config_entry.config_source == ConfigSource::TopicConfig as i8 { Override(Box::from(item)) } else { item };
                let item = if topic_info.configs_marked_deleted.contains(&config_entry.config_name) { Deleted(Box::from(item)) } else { item };
                let item = if topic_info.configs_marked_modified.contains(&config_entry.config_name) { Modified(Box::from(item)) } else { item };
                item
            })
            .collect::<Vec<TopicConfigurationItem>>();

        (SelectableList { list: list_items }).display(screen, (start_x, start_y + 3), width);
    }
}

fn show_user_input(screen: &mut impl Write, (_width, height): (u16, u16), user_input: Option<&String>) {
    write!(screen, "{}", cursor::Goto(1, height)).unwrap();
    match user_input {
        None => write!(screen, "{}", clear::CurrentLine).unwrap(),
        Some(input) => write!(screen, "{}{}", clear::CurrentLine, input).unwrap(),
    }
}
