use kafka_protocol::protocol_responses::metadata_response::TopicMetadata;
use std::io::{Stdout, stdout, Write};
use termion::{color, cursor, style};

pub struct SelectableList<A>
    where A: SelectableListItem {
    pub list: Vec<A>
}

impl<A> SelectableList<A>
    where A: SelectableListItem {
    pub fn display(&self, screen: &mut impl Write, (start_x, start_y): (u16, u16)) {
        write!(screen, "{}{}", cursor::Goto(start_x, start_y), style::Reset).unwrap();
        self.list.iter().for_each(|list_item| {
            write!(screen, "{}{}", list_item.display(), style::Reset).unwrap();
            write!(screen, "{}{}", cursor::Left(list_item.label().len() as u16), cursor::Down(1)).unwrap();
        });
        write!(screen, "{}{}", cursor::Goto(start_x, start_y), style::Reset).unwrap();
    }
}

pub trait SelectableListItem {
    fn display(&self) -> String;
    fn label(&self) -> &String;
}

pub enum ListItem {
    Normal(String),
    Selected(String),
    Deleted(String),
}

impl SelectableListItem for ListItem {
    fn display(&self) -> String {
        match &self {
            ListItem::Normal(label) => format!("{}{}", color::Fg(color::Cyan), &label),
            ListItem::Selected(label) => format!("{}{}{}", color::Fg(color::Black), color::Bg(color::White), &label),
            ListItem::Deleted(label) => format!("{}{}{}", color::Fg(color::Green), color::Bg(color::Red), &label)
        }
    }
    fn label(&self) -> &String {
        match &self {
            ListItem::Normal(label) => &label,
            ListItem::Selected(label) => &label,
            ListItem::Deleted(label) => &label
        }
    }
}