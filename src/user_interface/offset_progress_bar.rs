pub fn new(offset: i64, max_offset: i64, width: i64) -> String {
    let offset = offset.max(0);
    let max_offset = max_offset.max(1);

    let acc = |a: String, b: &str| format!("{}{}", a, b);

    let block_count = ((offset as f64 / max_offset as f64) * width as f64) as i64;
    let blocks = (0..block_count).map(|offset| { "█" }).fold(String::from(""), acc);
    let spaces = (0..(width - (block_count as i64))).map(|offset| { "_" }).fold(String::from(""), acc);

    format!("[{}{}]", blocks, spaces)
}