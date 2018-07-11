use kafka_protocol::protocol_responses::metadata_response::PartitionMetadata;
use std::io::{Stdout, stdout, Write};
use termion::{color, cursor, style};
use user_interface::offset_progress_bar;
use util::utils::VecToCSV;

pub struct SelectableList<A>
    where A: SelectableListItem {
    pub list: Vec<A>
}

impl<A> SelectableList<A>
    where A: SelectableListItem {
    pub fn display(&self, screen: &mut impl Write, (start_x, start_y): (u16, u16), width: u16) {
        write!(screen, "{}{}", cursor::Goto(start_x, start_y), style::Reset).unwrap();
        self.list.iter().for_each(|list_item| {
            let display = list_item.display();
            write!(screen, "{}{}", display, style::Reset).unwrap();
            write!(screen, "{}{}", cursor::Left(width), cursor::Down(1)).unwrap();
        });
        write!(screen, "{}{}", cursor::Goto(start_x, start_y), style::Reset).unwrap();
    }
}

pub trait SelectableListItem {
    fn display(&self) -> String;
}

pub enum TopicListItem {
    Normal(String, usize),
    Internal(String, usize),
    Deleted(String, usize),
    Selected(Box<TopicListItem>),
}

impl SelectableListItem for TopicListItem {
    fn display(&self) -> String {
        match &self {
            TopicListItem::Normal(label, partitions) => format!("{}{} [{}{}{}]", color::Fg(color::Cyan), &label, color::Fg(color::LightYellow), partitions, color::Fg(color::Cyan)),
            TopicListItem::Internal(label, partitions) =>
                format!("{}{}{} [{}{}{}]", color::Fg(color::LightMagenta), &label, color::Fg(color::Cyan), color::Fg(color::LightYellow), partitions, color::Fg(color::Cyan)),
            TopicListItem::Deleted(label, partitions) => format!("{}{}{} [{}]", color::Fg(color::Black), color::Bg(color::LightRed), &label, partitions),
            TopicListItem::Selected(topic_list_item) => format!("{}{}", color::Bg(color::LightBlack), topic_list_item.display())
        }
    }
}

pub enum PartitionListItem {
    Normal { partition: i32, partition_metadata: PartitionMetadata, consumer_offset: i64, partition_offset: i64 },
    Selected(Box<PartitionListItem>),
}

impl SelectableListItem for PartitionListItem {
    fn display(&self) -> String {
        use self::PartitionListItem::*;
        match &self {
            Normal { partition, partition_metadata, consumer_offset, partition_offset } => {
                format!("{}▶ {}{:<4} {}{}{} C:{:10} OF:{:10} L:{} R:{} ISR:{} O:{}{}",
                        color::Fg(color::LightYellow), color::Fg(color::Cyan),
                        partition,
                        color::Fg(color::Green), offset_progress_bar::new(*consumer_offset, *partition_offset, 10), color::Fg(color::Cyan),
                        if *consumer_offset > 0 { format!("{}", consumer_offset) } else { String::from("--") },
                        format!("{}", partition_offset),
                        partition_metadata.leader,
                        partition_metadata.replicas.as_csv(),
                        partition_metadata.isr.as_csv(),
                        color::Fg(color::LightRed), if !partition_metadata.offline_replicas.is_empty() { partition_metadata.offline_replicas.as_csv() } else { String::from("--") }
                )
            }
            Selected(item) => format!("{}{}", color::Bg(color::LightBlack), item.display())
        }
    }
}

pub enum TopicConfigurationItem {
    Config { name: String, value: Option<String> },
    NewConfig { name: String, value: Option<String> },
    Selected(Box<TopicConfigurationItem>),
    ReadOnlyConfig(Box<TopicConfigurationItem>),
    SensitiveConfig(Box<TopicConfigurationItem>),
}

impl SelectableListItem for TopicConfigurationItem {
    fn display(&self) -> String {
        use self::TopicConfigurationItem::*;
        match &self {
            Config { name, value } => format!("{}: {}", name, value.as_ref().unwrap_or(&format!(""))),
            NewConfig { name, value } => format!("{}{}: {}", color::Fg(color::LightRed),  name, value.as_ref().unwrap_or(&format!(""))),
            Selected(config) => format!("{}{}", color::Bg(color::LightBlack), config.display()),
            ReadOnlyConfig(config) => format!("{}{}", color::Fg(color::LightMagenta), config.display()),
            SensitiveConfig(config) => format!("{}{}", color::Fg(color::LightRed), config.display()),
        }
    }
}