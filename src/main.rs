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

pub mod utils;
pub mod kafka_protocol;
pub mod tcp_stream_util;
pub mod app_config;
pub mod event_bus;
pub mod state;
pub mod ui;

fn main() {
    let args: Vec<String> = env::args().collect();
    let app_config = app_config::from(&args);
    let sender = event_bus::start();

    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = stdin();
    ui::clear_screen();

    let bootstrap_server = || BootstrapServer(String::from(app_config.bootstrap_server));
    sender.send(Message::GetTopics(bootstrap_server()));

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => {
                ui::clear_screen();
                break;
            },
            Key::Char('r') => {
                sender.send(Message::GetTopics(bootstrap_server()));
            },
            Key::Char('d') => {
                sender.send(Message::DeleteTopic(bootstrap_server()));
            },
            Key::Up => {
                sender.send(Message::SelectTopic(Up));
            },
            Key::Down => {
                sender.send(Message::SelectTopic(Down));
            },
            Key::Char('i') => {
                sender.send(Message::ToggleTopicInfo());
            },
            _ => {}
        }
    }
}