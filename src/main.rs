extern crate byteorder;
#[macro_use]
extern crate proptest;
extern crate termion;

use app_config::AppConfig;
use event_bus::Message;
use event_bus::BootstrapServer;
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
use std::sync::mpsc::Sender;
use termion::{color, style};

pub mod kafka_protocol;
pub mod tcp_stream_util;
pub mod app_config;
pub mod event_bus;

fn main() {
    let args: Vec<String> = env::args().collect();
    let app_config = app_config::from(&args);
    let sender = event_bus::start();

    sender.send(Message::GetTopics(BootstrapServer(String::from(app_config.bootstrap_server))));

    std::thread::sleep_ms(5000);
}



