use std::cmp;
use std::fmt::Display;

pub fn pad_right(input: &String, width: u16) -> String {
    let str_len = cmp::min(input.len() as u16, width);
    let pad_length = cmp::max(width - str_len, 0);
    (0..pad_length).map(|_i| String::from(" ")).fold(input.clone(), |a, b| format!("{}{}", a, b))
}

pub fn bool_yes_no(b: bool) -> String {
    match b {
        true => String::from("Yes"),
        false => String::from("No"),
    }
}

pub fn to_hex_array(bytes: &Vec<u8>) -> Vec<String> {
    bytes.iter().cloned().map(|b| format!("{:02X}", b)).collect::<Vec<String>>()
}

pub fn current_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Failed to get current time");
    (since_the_epoch.as_secs() * 1000) + (since_the_epoch.subsec_nanos() as u64 / 1_000_000)
}

pub trait VecToCSV {
    fn as_csv(&self) -> String;
}

impl<T: Display> VecToCSV for Vec<T> {
    fn as_csv(&self) -> String {
        let fold = |a: String, b: &T| {
            if a.is_empty() {
                format!("{}", b)
            } else {
                format!("{},{}", a, b)
            }
        };
        format!("{}", self.iter().fold(String::from(""), fold))
    }
}

pub trait Flatten<A> {
    fn flatten(self) -> Vec<A>;
}

impl<A> Flatten<A> for Vec<Vec<A>> {
    fn flatten(self) -> Vec<A> {
        let init = vec![];
        self.into_iter().fold(init, |mut acc, mut a| {
            acc.append(&mut a);
            acc
        })
    }
}
