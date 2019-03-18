extern crate clap;
extern crate regex;
extern crate termion;
extern crate topiks_kafka_client;

use std::env;
use std::io::stdin;

use api_client::ApiClient;
use api_client::ApiClientTrait;
use api_client::ApiRequestError;
use clap::{App, Arg};
use kafka_protocol::protocol_request::*;
use kafka_protocol::protocol_requests::findcoordinator_request::CoordinatorType;
use kafka_protocol::protocol_requests::findcoordinator_request::FindCoordinatorRequest;
use kafka_protocol::protocol_response::*;
use kafka_protocol::protocol_responses::findcoordinator_response::FindCoordinatorResponse;
use regex::Regex;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::terminal_size;
use topiks_kafka_client::*;

use event_bus::ConsumerGroup;
use event_bus::Creation;
use event_bus::Message;
use event_bus::MoveSelection::*;
use event_bus::TopicQuery::*;
use state::CurrentView;
use state::DialogMessage;
use user_interface::user_input;

pub mod error_codes;
pub mod event_bus;
pub mod state;
pub mod user_interface;
pub mod util;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

struct AppConfig<'a> {
    bootstrap_server: KafkaServerAddr,
    consumer_group: Option<&'a str>,
    request_timeout_ms: i32,
    deletion_allowed: bool,
    deletion_confirmation: bool,
    modification_enabled: bool,
}

fn main() -> Result<(), u8> {
    let _args: Vec<String> = env::args().collect();

    let matches = App::new("topiks")
        .version(VERSION)
        .arg(Arg::with_name("bootstrap-server").required(true).takes_value(true).help("A single Kafka broker [DOMAIN|IP]:PORT"))
        .arg(Arg::with_name("tls").long("tls").required(false).help("Enable TLS"))
        .arg(Arg::with_name("consumer-group").long("consumer-group").short("c").takes_value(true).help("Consumer group for fetching offsets"))
        .arg(Arg::with_name("delete").short("D").help("Enable topic/config deletion"))
        .arg(Arg::with_name("no-delete-confirmation").long("no-delete-confirmation").help("Disable delete confirmation <Danger!>"))
        .arg(Arg::with_name("modify").short("M").help("Enable creation of topics and modification of topic configurations"))
        .get_matches();

    let enable_tls = matches.is_present("tls");
    let bootstrap_server =
        KafkaServerAddr::from_arg(matches.value_of("bootstrap-server").unwrap(), enable_tls).ok_or(error_codes::COULD_NOT_PARSE_BOOTSTRAP_SERVER)?;

    let app_config = AppConfig {
        bootstrap_server,
        consumer_group: matches.value_of("consumer-group"),
        request_timeout_ms: 300_000, // 5 minutes
        deletion_allowed: matches.is_present("delete"),
        deletion_confirmation: !matches.is_present("no-delete-confirmation"),
        modification_enabled: matches.is_present("modify"),
    };

    if let Err(err) =
        kafka_protocol::api_verification::apply(ApiClient::new(), &app_config.bootstrap_server, &kafka_protocol::api_verification::apis_in_use())
    {
        eprintln!("Kafka Protocol API Error(s): {:?}", err);
        Err(error_codes::KAFKA_API_VERIFICATION_FAIL)
    } else {
        let sender = event_bus::start();
        let _stdout = &mut AlternateScreen::from(std::io::stdout().into_raw_mode().unwrap()); // raw mode to avoid screen output
        let stdin = stdin();

        let bootstrap_server = || app_config.bootstrap_server.clone();

        let consumer_group = app_config.consumer_group.clone().and_then(|cg| {
            let find_coordinator_response: Result<Response<FindCoordinatorResponse>, ApiRequestError> = ApiClient::new().request(
                &app_config.bootstrap_server,
                Request::of(FindCoordinatorRequest { coordinator_key: String::from(cg), coordinator_type: CoordinatorType::Group as i8 }),
            );
            find_coordinator_response.ok().and_then(|find_coordinator_response| {
                if find_coordinator_response.response_message.error_code == 0 {
                    Some(ConsumerGroup(String::from(cg), find_coordinator_response.response_message.coordinator))
                } else {
                    sender
                        .send(Message::DisplayUIMessage(DialogMessage::Error(format!("Could not determine coordinator for consumer group {}", cg))))
                        .unwrap();
                    None
                }
            })
        });

        sender.send(Message::GetMetadata(bootstrap_server(), consumer_group.clone())).unwrap();

        let create_topic_regex = Regex::new(r"^[A-Za-z0-9\._]{1, 249}:[0-9]{1,3}:[0-9]{1,3}$").expect("Could not compile regex for creating topics");

        for key in stdin.keys() {
            match key.unwrap() {
                Key::Char('h') => {
                    sender.send(Message::ToggleView(CurrentView::HelpScreen)).unwrap();
                }
                Key::Char('q') => match sender.send(Message::Quit) {
                    Ok(_) => break,
                    Err(_) => {
                        eprintln!("Failed to signal event bus/user interface thread. Exiting now");
                        break;
                    }
                },
                Key::Char('r') => {
                    sender.send(Message::DisplayUIMessage(DialogMessage::None)).unwrap();
                    sender.send(Message::GetMetadata(bootstrap_server(), consumer_group.clone())).unwrap();
                }
                Key::Char('c') => {
                    if app_config.modification_enabled {
                        let (_, height) = terminal_size().unwrap();
                        match user_input::read("▶︎ ", (1, height), sender.clone()) {
                            Ok(Some(ref input)) if create_topic_regex.is_match(input.as_str()) => {
                                match input.split(":").collect::<Vec<&str>>().as_slice() {
                                    &[topic, partitions, replication_factor] => {
                                        let create_topic = partitions.to_string().parse::<i32>().and_then(|partitions| {
                                            replication_factor.to_string().parse::<i16>().map(|replication_factor| Creation::Topic {
                                                name: topic.to_string(),
                                                partitions,
                                                replication_factor,
                                            })
                                        });
                                        match create_topic {
                                            Ok(create_topic) => {
                                                sender.send(Message::Create(bootstrap_server(), create_topic, app_config.request_timeout_ms)).unwrap()
                                            }
                                            Err(_) => sender
                                                .send(Message::DisplayUIMessage(DialogMessage::Error("Invalid input for creating topic".to_string())))
                                                .unwrap(),
                                        }
                                    }
                                    _ => sender
                                        .send(Message::DisplayUIMessage(DialogMessage::Error("Invalid input for creating topic".to_string())))
                                        .unwrap(),
                                }
                            }
                            _ => sender
                                .send(Message::DisplayUIMessage(DialogMessage::Error(
                                    "Input should be [topic]:[partitions]:[replication factor]".to_string(),
                                )))
                                .unwrap(),
                        }
                    }
                }
                Key::Char('d') => {
                    if app_config.deletion_allowed {
                        sender.send(Message::DisplayUIMessage(DialogMessage::Warn(format!("Deleting...")))).unwrap();
                        if app_config.deletion_confirmation {
                            let (_, height) = terminal_size().unwrap();
                            match user_input::read("[Yes]?: ", (1, height), sender.clone()) {
                                Ok(Some(confirm)) => {
                                    if confirm.eq("Yes") {
                                        sender.send(Message::Delete(bootstrap_server(), app_config.request_timeout_ms)).unwrap();
                                    } else {
                                        sender.send(Message::Noop).unwrap();
                                    }
                                }
                                _ => (),
                            }
                        } else {
                            sender.send(Message::Delete(bootstrap_server(), app_config.request_timeout_ms)).unwrap();
                        }
                        sender.send(Message::DisplayUIMessage(DialogMessage::None)).unwrap();
                    }
                }
                Key::Up => {
                    sender.send(Message::Select(Up)).unwrap();
                }
                Key::PageUp => {
                    sender.send(Message::Select(PageUp)).unwrap();
                }
                Key::Down => {
                    sender.send(Message::Select(Down)).unwrap();
                }
                Key::PageDown => {
                    sender.send(Message::Select(PageDown)).unwrap();
                }
                Key::Home => {
                    sender.send(Message::Select(Top)).unwrap();
                }
                Key::End => {
                    sender.send(Message::Select(Bottom)).unwrap();
                }
                Key::Char('i') => {
                    sender.send(Message::DisplayUIMessage(DialogMessage::None)).unwrap();
                    sender.send(Message::ToggleView(CurrentView::TopicInfo)).unwrap();
                    sender.send(Message::GetMetadata(bootstrap_server(), consumer_group.clone())).unwrap();
                }
                Key::Char('p') => {
                    sender.send(Message::DisplayUIMessage(DialogMessage::None)).unwrap();
                    sender.send(Message::ToggleView(CurrentView::Partitions)).unwrap();
                    sender.send(Message::GetMetadata(bootstrap_server(), consumer_group.clone())).unwrap();
                }
                Key::Char('/') => {
                    sender.send(Message::DisplayUIMessage(DialogMessage::Info(format!("Search")))).unwrap();
                    let (_width, height) = terminal_size().unwrap();
                    let query = match user_input::read("/", (1, height), sender.clone()) {
                        Ok(Some(query)) => Message::SetTopicQuery(Query(query)),
                        Ok(None) => Message::SetTopicQuery(NoQuery),
                        Err(_) => Message::SetTopicQuery(NoQuery),
                    };
                    sender.send(query).unwrap();
                    sender.send(Message::Select(SearchNext)).unwrap();
                    sender.send(Message::DisplayUIMessage(DialogMessage::None)).unwrap();
                }
                Key::Char('n') => {
                    // TODO support Shift+n for reverse
                    sender.send(Message::Select(SearchNext)).unwrap();
                }
                Key::Char(':') => {
                    if app_config.modification_enabled {
                        let (_width, height) = terminal_size().unwrap();
                        match user_input::read(":", (1, height), sender.clone()) {
                            Ok(modify_value) => {
                                sender.send(Message::ModifyValue(bootstrap_server(), modify_value)).unwrap();
                            }
                            _ => (),
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
