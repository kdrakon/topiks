use std::cmp;
use termion::screen::AlternateScreen;
use std::io::Stdout;
use utils;

pub fn pad_right(input: &String, width: u16) -> String {
    let pad_length = cmp::max(width - input.len() as u16, 0);
    (0..pad_length).map(|i| { String::from(" ") }).fold(input.clone(), |a, b| format!("{}{}", a, b))
}

pub fn bool_yes_no(b: bool) -> String {
    match b {
        true => String::from("Yes"),
        false => String::from("No")
    }
}

pub fn to_hex_array(bytes: &Vec<u8>) -> Vec<String> {
    bytes.iter().cloned().map(|b|{format!("{:02X}", b)}).collect::<Vec<String>>()
}

pub fn current_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Failed to get current time");
    (since_the_epoch.as_secs() * 1000) + (since_the_epoch.subsec_nanos() as u64 / 1_000_000)
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