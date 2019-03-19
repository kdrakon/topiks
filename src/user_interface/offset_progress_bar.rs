pub fn new(offset: i64, max_offset: i64, width: i64) -> String {
    let offset = offset.max(0);
    let max_offset = max_offset.max(1);

    let whole_blocks = (offset * width) / max_offset;
    let last_block = match whole_blocks {
        0 => None,
        n if n == width => None,
        _ => Some(((offset * width) % max_offset) as f64 / max_offset as f64),
    };
    let empty_blocks = width - whole_blocks - if last_block.is_some() { 1 } else { 0 };

    format!(
        "[{}{}{}]",
        vec![WHOLE; whole_blocks as usize].join(""),
        last_block.map(last_block_str).unwrap_or(""),
        vec![EMPTY; empty_blocks as usize].join("")
    )
}

const EMPTY: &str = "░";
const WHOLE: &str = "█";
const ONE_EIGHTH: &str = "▏";
const ONE_QUARTER: &str = "▎";
const THREE_EIGHTHS: &str = "▍";
const ONE_HALF: &str = "▌";
const FIVE_EIGHTHS: &str = "▋";
const THREE_QUARTERS: &str = "▊";
const SEVEN_EIGHTHS: &str = "▉";

fn last_block_str(percent: f64) -> &'static str {
    if percent == 0_f64 {
        EMPTY
    } else if percent <= 0.125_f64 {
        ONE_EIGHTH
    } else if percent <= 0.25_f64 {
        ONE_QUARTER
    } else if percent <= 0.375_f64 {
        THREE_EIGHTHS
    } else if percent <= 0.50_f64 {
        ONE_HALF
    } else if percent <= 0.625_f64 {
        FIVE_EIGHTHS
    } else if percent <= 0.75_f64 {
        THREE_QUARTERS
    } else if percent <= 0.875_f64 {
        SEVEN_EIGHTHS
    } else {
        WHOLE
    }
}
