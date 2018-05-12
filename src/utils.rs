use std::cmp;

pub fn pad_right(input: &String, width: u16) -> String {
    let pad_length = cmp::max(width - input.len() as u16, 0);
    (0..pad_length).map(|i| { String::from(" ") }).fold(input.clone(), |a, b| format!("{}{}", a, b))
}
