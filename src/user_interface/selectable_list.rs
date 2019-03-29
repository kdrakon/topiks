use std::io::Write;

use termion::{clear, color, cursor, style};

use crate::kafka_protocol::protocol_responses::metadata_response::PartitionMetadata;
use crate::user_interface::offset_progress_bar;
use crate::util::utils::VecToCSV;

pub struct SelectableList<A>
where
    A: SelectableListItem,
{
    pub list: Vec<A>,
}

impl<A> SelectableList<A>
where
    A: SelectableListItem,
{
    pub fn display(&self, screen: &mut impl Write, (start_x, start_y): (u16, u16), height: u16) {
        let blank_items = vec![format!("{}{}", clear::CurrentLine, cursor::Down(1)); height as usize - self.list.len()].join("");

        let list_items = self
            .list
            .iter()
            .map(|list_item| {
                let display = list_item.display();
                format!("{}{}{}{}{}", display, style::Reset, clear::UntilNewline, cursor::Left(display.len() as u16), cursor::Down(1))
            })
            .collect::<Vec<String>>()
            .join("");

        write!(
            screen,
            "{cursor_to_start}{style_reset}{list_items}{blank_items}{style_reset}",
            cursor_to_start = cursor::Goto(start_x, start_y),
            style_reset = style::Reset,
            list_items = list_items,
            blank_items = blank_items
        )
        .unwrap();
    }
}

pub trait SelectableListItem {
    fn display(&self) -> String;
}

pub enum TopicListItem<'a> {
    Normal(&'a str, usize),
    Internal(&'a str, usize),
    Deleted(&'a str, usize),
    Selected(Box<TopicListItem<'a>>),
}

impl<'a> SelectableListItem for TopicListItem<'a> {
    fn display(&self) -> String {
        match &self {
            TopicListItem::Normal(label, partitions) => {
                format!("{}{} [{}{}{}]", color::Fg(color::Cyan), &label, color::Fg(color::LightYellow), partitions, color::Fg(color::Cyan))
            }
            TopicListItem::Internal(label, partitions) => format!(
                "{}{}{} [{}{}{}]",
                color::Fg(color::LightMagenta),
                &label,
                color::Fg(color::Cyan),
                color::Fg(color::LightYellow),
                partitions,
                color::Fg(color::Cyan)
            ),
            TopicListItem::Deleted(label, partitions) => {
                format!("{}{}{} [{}]", color::Fg(color::Black), color::Bg(color::LightRed), &label, partitions)
            }
            TopicListItem::Selected(topic_list_item) => format!("{}{}", color::Bg(color::LightBlack), topic_list_item.display()),
        }
    }
}

pub enum PartitionListItem<'a> {
    Normal { partition: i32, partition_metadata: &'a PartitionMetadata, consumer_offset: i64, partition_offset: i64 },
    Selected(Box<PartitionListItem<'a>>),
}

impl<'a> SelectableListItem for PartitionListItem<'a> {
    fn display(&self) -> String {
        use self::PartitionListItem::*;
        match &self {
            Normal { partition, partition_metadata, consumer_offset, partition_offset } => format!(
                "{}▶ {}{:<4} {}{}{} C:{:10} OF:{:10} L:{} R:{} ISR:{} O:{}{}",
                color::Fg(color::LightYellow),
                color::Fg(color::White),
                partition,
                color::Fg(color::Green),
                offset_progress_bar::new(*consumer_offset, *partition_offset, 50),
                color::Fg(color::White),
                if *consumer_offset > 0 { format!("{}", consumer_offset) } else { String::from("--") },
                format!("{}", partition_offset),
                partition_metadata.leader,
                partition_metadata.replicas.as_csv(),
                partition_metadata.isr.as_csv(),
                color::Fg(color::LightRed),
                if !partition_metadata.offline_replicas.is_empty() { partition_metadata.offline_replicas.as_csv() } else { String::from("--") }
            ),
            Selected(item) => format!("{}{}", color::Bg(color::LightBlack), item.display()),
        }
    }
}

pub enum TopicConfigurationItem {
    Config { name: String, value: Option<String> },
    Selected(Box<TopicConfigurationItem>),
    Override(Box<TopicConfigurationItem>),
    Deleted(Box<TopicConfigurationItem>),
    Modified(Box<TopicConfigurationItem>),
}

impl SelectableListItem for TopicConfigurationItem {
    fn display(&self) -> String {
        use self::TopicConfigurationItem::*;
        match &self {
            Config { name, value } => format!("{}: {}", name, value.as_ref().unwrap_or(&format!(""))),
            Selected(config) => format!("{}{}", color::Bg(color::LightBlack), config.display()),
            Override(config) => format!("{}{}", color::Fg(color::LightMagenta), config.display()),
            Deleted(config) => format!("{}{} {}", color::Bg(color::LightBlue), config.display(), "[refresh]"),
            Modified(config) => format!("{}{} {}", color::Bg(color::LightBlue), config.display(), "[refresh]"),
        }
    }
}
