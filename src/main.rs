extern crate byteorder;
extern crate clap;
#[cfg(test)]
#[macro_use]
extern crate proptest;
extern crate termion;

use clap::{App, Arg};
use event_bus::BootstrapServer;
use event_bus::ConsumerGroup;
use event_bus::Message;
use event_bus::MoveSelection::*;
use event_bus::TopicQuery::*;
use kafka_protocol::protocol_request::*;
use kafka_protocol::protocol_requests::deletetopics_request::*;
use kafka_protocol::protocol_requests::findcoordinator_request::CoordinatorType;
use kafka_protocol::protocol_requests::findcoordinator_request::FindCoordinatorRequest;
use kafka_protocol::protocol_requests::metadata_request::*;
use kafka_protocol::protocol_response::*;
use kafka_protocol::protocol_responses::deletetopics_response::*;
use kafka_protocol::protocol_responses::findcoordinator_response::FindCoordinatorResponse;
use kafka_protocol::protocol_responses::metadata_response::*;
use kafka_protocol::protocol_responses::metadata_response::MetadataResponse;
use kafka_protocol::protocol_serializable::*;
use state::UIMessage;
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
use util::tcp_stream_util::TcpRequestError;

pub mod util;
pub mod kafka_protocol;
pub mod event_bus;
pub mod state;
pub mod user_interface;

struct AppConfig<'a> {
    bootstrap_server: &'a str,
    consumer_group: Option<&'a str>,
    request_timeout_ms: i32,
    topic_deletion: bool,
    topic_deletion_confirmation: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let matches = App::new("topiks")
        .version("1.1.0")
        .arg(Arg::with_name("bootstrap-server").required(true).takes_value(true).help("A single Kafka broker to connect to"))
        .arg(Arg::with_name("consumer-group").long("consumer-group").short("c").takes_value(true).help("Consumer group for fetching offsets"))
        .arg(Arg::with_name("delete").short("D").help("Enable topic deletion"))
        .arg(Arg::with_name("no-delete-confirmation").long("no-delete-confirmation").help("Disable delete confirmation <Danger!>"))
        .get_matches();

    let app_config = AppConfig {
        bootstrap_server: matches.value_of("bootstrap-server").unwrap(),
        consumer_group: matches.value_of("consumer-group"),
        request_timeout_ms: 30_000,
        topic_deletion: matches.is_present("delete"),
        topic_deletion_confirmation: !matches.is_present("no-delete-confirmation"),
    };

    let sender = event_bus::start();
    let stdout = &mut AlternateScreen::from(std::io::stdout().into_raw_mode().unwrap()); // raw mode to avoid screen output
    let stdin = stdin();

    let bootstrap_server = || BootstrapServer(String::from(app_config.bootstrap_server));

    let consumer_group = app_config.consumer_group.clone().and_then(|cg| {
        let find_coordinator_response: Result<Response<FindCoordinatorResponse>, TcpRequestError> =
            util::tcp_stream_util::request(app_config.bootstrap_server,
                                           Request::of(
                                               FindCoordinatorRequest { coordinator_key: String::from(cg), coordinator_type: CoordinatorType::Group as i8 },
                                               10,
                                               1,
                                           ),
            );
        find_coordinator_response.ok().and_then(|find_coordinator_response| {
            if find_coordinator_response.response_message.error_code == 0 {
                Some(ConsumerGroup(String::from(cg), find_coordinator_response.response_message.coordinator))
            } else {
                sender.send(Message::DisplayUIMessage(UIMessage::Error(format!("Could not determine coordinator for consumer group {}", cg))));
                None
            }
        })
    });

    sender.send(Message::GetTopics(bootstrap_server()));

    for key in stdin.keys() {
        match key.unwrap() {
            Key::Char('q') => {
                break;
            }
            Key::Char('r') => {
                sender.send(Message::DisplayUIMessage(UIMessage::None));
                sender.send(Message::GetTopics(bootstrap_server()));
            }
            Key::Char('d') => {
                if app_config.topic_deletion {
                    sender.send(Message::DisplayUIMessage(UIMessage::Warn(format!("Deleting topic"))));
                    if app_config.topic_deletion_confirmation {
                        let (width, height) = terminal_size().unwrap();
                        if let Some(ref confirm) = user_input::read("[Yes]?: ", (1, height), sender.clone()) {
                            if confirm.eq("Yes") {
                                sender.send(Message::DeleteTopic(bootstrap_server()));
                            } else {
                                sender.send(Message::Noop);
                            }
                        }
                    } else {
                        sender.send(Message::DeleteTopic(bootstrap_server()));
                    }
                    sender.send(Message::DisplayUIMessage(UIMessage::None));
                }
            }
            Key::Up => {
                sender.send(Message::Select(Up));
            }
            Key::Down => {
                sender.send(Message::Select(Down));
            }
            Key::Home => {
                sender.send(Message::Select(Top));
            }
            Key::End => {
                sender.send(Message::Select(Bottom));
            }
            Key::Char('i') => {
                sender.send(Message::ToggleTopicInfo(bootstrap_server()));
            }
            Key::Char('p') => {
                sender.send(Message::TogglePartitionInfo(bootstrap_server(), consumer_group.clone()));
            }
            Key::Char('/') => {
                sender.send(Message::DisplayUIMessage(UIMessage::Info(format!("Search"))));
                let (width, height) = terminal_size().unwrap();
                let query = match user_input::read("/", (1, height), sender.clone()) {
                    Some(query) => Message::SetTopicQuery(Query(query)),
                    None => Message::SetTopicQuery(NoQuery)
                };
                sender.send(query);
                sender.send(Message::Select(SearchNext));
                sender.send(Message::DisplayUIMessage(UIMessage::None));
            }
            Key::Char('n') => { // TODO support Shift+n for reverse
                sender.send(Message::Select(SearchNext));
            }
            _ => {}
        }
    }
}