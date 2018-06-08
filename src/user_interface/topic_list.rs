use termion::screen::AlternateScreen;
use termion::{cursor, color, style};
use std::io::{Stdout, stdout, Write};
use kafka_protocol::protocol_responses::metadata_response::TopicMetadata;
use termion::raw::RawTerminal;
use termion::raw::IntoRawMode;

pub struct TopicList<A>
    where A: TopicListItem {
    pub list: Vec<A>
}

impl<A> TopicList<A>
    where A: TopicListItem {
    pub fn display(&self, screen: &mut AlternateScreen<RawTerminal<Stdout>>, (start_x, start_y): (u16, u16)) {
        write!(screen, "{}{}", cursor::Goto(start_x, start_y), style::Reset).unwrap();
        self.list.iter().for_each(|list_item| {
            write!(screen, "{}{}", list_item.display(), style::Reset).unwrap();
            write!(screen, "{}{}", cursor::Left(list_item.label().len() as u16), cursor::Down(1)).unwrap();
        });
        write!(screen, "{}{}", cursor::Goto(start_x, start_y), style::Reset).unwrap();
    }
}

pub trait TopicListItem {
    fn display(&self) -> String;
    fn label(&self) -> &String;
}

pub enum ListItem {
    Normal(String),
    Selected(String),
    Deleted(String),
}

impl TopicListItem for ListItem {
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

pub struct PagedVec<'a, A: 'a> {
    indexes: usize,
    page_length: usize,
    pages: Vec<Vec<&'a A>>,
}

impl<'a, A> PagedVec<'a, A> {
    pub fn from(vec: &'a Vec<A>, page_length: usize) -> PagedVec<'a, A> {
        PagedVec {
            indexes: vec.len(),
            page_length,
            pages: vec.chunks(page_length).map(|slice| {
                slice.iter().collect::<Vec<&'a A>>()
            }).collect::<Vec<Vec<&'a A>>>(),
        }
    }

    pub fn page(&'a self, index: usize) -> Option<(usize, &'a Vec<&'a A>)> {
        self.pages.get((index as f32 / self.page_length as f32).floor() as usize).map(|page| {
            (
                index % self.page_length,
                page
            )
        })
    }
}