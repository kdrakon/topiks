use std;
use std::sync::mpsc::Sender;

use termion::event::Key;
use termion::input::TermRead;

use crate::event_bus::Message;
use crate::event_bus::Message::UserInput;

pub fn read(label: &str, (_cursor_x, _cursor_y): (u16, u16), sender: Sender<Message>) -> Result<Option<String>, ()> {
    let stdin = std::io::stdin();
    sender.send(UserInput(String::from(label))).unwrap();

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

        sender.send(UserInput(format!("{}{}", label, input.iter().collect::<String>()))).unwrap();
    }

    sender.send(UserInput(String::from(""))).unwrap();

    if !cancelled {
        let read = input.iter().collect::<String>();
        match read.len() {
            0 => Ok(None),
            _ => Ok(Some(read)),
        }
    } else {
        Err(())
    }
}
