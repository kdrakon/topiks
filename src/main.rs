extern crate byteorder;
#[macro_use]
extern crate proptest;
extern crate termion;

use app_config::AppConfig;
use event_bus::BootstrapServer;
use event_bus::Message;
use event_bus::MoveSelection::*;
use kafka_protocol::protocol_request::*;
use kafka_protocol::protocol_requests::deletetopics_request::*;
use kafka_protocol::protocol_requests::metadata_request::*;
use kafka_protocol::protocol_response::*;
use kafka_protocol::protocol_responses::deletetopics_response::*;
use kafka_protocol::protocol_responses::metadata_response::*;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use kafka_protocol::protocol_serializable::*;
use std::env;
use std::io;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc::Sender;
use termion::{color, style};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::terminal_size;
use user_interface::user_input;
use event_bus::TopicQuery::*;

pub mod utils;
pub mod kafka_protocol;
pub mod tcp_stream_util;
pub mod app_config;
pub mod event_bus;
pub mod state;
pub mod user_interface;

fn main() {
    let args: Vec<String> = env::args().collect();
    let app_config = app_config::from(&args);
    let sender = event_bus::start();

    let mut stdout = std::io::stdout().into_raw_mode().unwrap(); // raw mode to avoid screen output
    let screen = &mut AlternateScreen::from(std::io::stdout());
    let stdin = stdin();

    let bootstrap_server = || BootstrapServer(String::from(app_config.bootstrap_server));
    sender.send(Message::GetTopics(bootstrap_server()));

    for key in stdin.keys() {
        match key.unwrap() {
            Key::Char('q') => {
                write!(screen, "{}", termion::clear::All);
                screen.flush().unwrap();
                break;
            }
            Key::Char('r') => {
                sender.send(Message::GetTopics(bootstrap_server()));
            }
            Key::Char('d') => {
                sender.send(Message::DeleteTopic(bootstrap_server()));
            }
            Key::Up => {
                sender.send(Message::SelectTopic(Up));
            }
            Key::Down => {
                sender.send(Message::SelectTopic(Down));
            }
            Key::Home => {
                sender.send(Message::SelectTopic(Top));
            }
            Key::End => {
                sender.send(Message::SelectTopic(Bottom));
            }
            Key::Char('i') => {
                sender.send(Message::ToggleTopicInfo(bootstrap_server()));
            }
            Key::Char('/') => {
                let (width, height) = terminal_size().unwrap();
                let query = match user_input::read(screen,"/",(1, height)) {
                    Some(query) => Message::SetTopicQuery(Query(query)),
                    None => Message::SetTopicQuery(NoQuery)
                };
                sender.send(query);
                sender.send(Message::SelectTopic(SearchNext));
            }
            Key::Char('n') => { // TODO support Shift+n for reverse
                sender.send(Message::SelectTopic(SearchNext));
            }
            _ => {}
        }
    }
}