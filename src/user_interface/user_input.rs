use std;
use std::io::{stdin, stdout, Write};
use termion::{cursor, style};
use termion::event::Key;
use termion::input::TermRead;
use termion::screen::AlternateScreen;

pub fn read(cursor_symbol: &str, (cursor_x, cursor_y): (u16, u16)) -> Option<String> {

    let screen = &mut AlternateScreen::from(std::io::stdout());
    let stdin = std::io::stdin();

    let mut input: Vec<char> = vec![];
    for key in stdin.keys() {
        match key.unwrap() {
            Key::Backspace => {
                input.pop();
            }
            Key::Char('\n') => {
                break;
            },
            Key::Char(c) => {
                input.push(c);
            },
            _ => {} // ignore everything else
        }

        write!(screen, "{}{}{}", cursor::Goto(cursor_x, cursor_y), cursor_symbol, input.iter().collect::<String>());
        screen.flush().unwrap();
    }

    let read = input.iter().collect::<String>();
    match read.len() {
        0 => None,
        _ => Some(read)
    }
}