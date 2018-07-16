use std;
use std::io::{stdin, stdout, Write};
use std::io::Stdout;
use termion::{cursor, style, clear};
use termion::event::Key;
use termion::input::TermRead;
use termion::screen::AlternateScreen;
use termion::raw::IntoRawMode;
use std::sync::mpsc::Sender;
use event_bus::Message;
use event_bus::Message::UserInput;

pub fn read(label: &str, (cursor_x, cursor_y): (u16, u16), sender: Sender<Message>) -> Result<Option<String>, ()> {
    let stdin = std::io::stdin();
    sender.send(UserInput(String::from(label)));

    let mut input: Vec<char> = vec![];
    let mut cancelled = false;

    for key in stdin.keys() {
        match key.unwrap() {
            Key::Backspace => {
                input.pop();
            }
            Key::Char('\n') => {
                break;
            }
            Key::Char(c) => {
                input.push(c);
            }
            Key::Esc => {
                cancelled = true;
                break;
            }
            _ => {} // ignore everything else
        }

        sender.send(UserInput(format!("{}{}", label, input.iter().collect::<String>())));
    }

    sender.send(UserInput(String::from("")));

    if !cancelled {
        let read = input.iter().collect::<String>();
        match read.len() {
            0 => Ok(None),
            _ => Ok(Some(read))
        }
    } else {
        Err(())
    }
}