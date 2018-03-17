extern crate byteorder;
extern crate cursive;
#[macro_use]
extern crate proptest;

use cursive::Cursive;
use cursive::traits::*;
use cursive::views::*;
use kafka_protocol::protocol_request::*;
use kafka_protocol::protocol_response::*;
use kafka_protocol::protocol_responses::metadata_response::*;
use kafka_protocol::protocol_serializable::*;
use self::byteorder::{BigEndian, ReadBytesExt};
use std::io::*;
use std::net::TcpStream;

pub mod kafka_protocol;

fn main() {
    let metadata_request = RequestMessage::MetadataRequest { topics: None, allow_auto_topic_creation: false };

    let request =
        Request {
            header: RequestHeader {
                api_key: 3,
                api_version: 5,
                correlation_id: 42,
                client_id: String::from("sean"),
            },
            request_message: metadata_request,
        };

    let bytes = request.into_protocol_bytes().unwrap();

    let send_result =
        TcpStream::connect("localhost:9092").and_then(|mut stream| {
            stream.write(bytes.as_slice()).and_then(|_| {
                let mut result_size_buf: [u8; 4] = [0; 4];
                stream.read(&mut result_size_buf).and_then(|_| {
                    Cursor::new(result_size_buf.to_vec()).read_i32::<BigEndian>()
                }).and_then(|result_size| {
                    let mut message_buf: Vec<u8> = vec![0; result_size as usize];
                    stream.read_exact(&mut message_buf).map(|_| message_buf)
                })
            })
        });

    let response =
        match send_result {
            Ok(result) => {
                println!("printing results for {:?} bytes...", result.len());
                let result: ProtocolDeserializeResult<Response<MetadataResponse>> = result.into_protocol_type();
                let response = result.unwrap();
                println!("output: {:?}", response);
                response
            }
            Err(e) => {
                println!("failed, {:?}", e);
                unimplemented!()
            }
        };

    let mut cursive = Cursive::new();

    cursive.add_global_callback('q', |s| s.quit());

    let mut topics = SelectView::<TopicMetadata>::new().on_submit(topics_select_callback);

    response.response_message.topic_metadata.into_iter().for_each(|topic_metadata| {
        topics.add_item(topic_metadata.topic.clone(), topic_metadata)
    });

    let topic_info = TextView::new("select a topic").with_id("topic_info");

    let root_layout =
        LinearLayout::vertical()
            .child(BoxView::with_full_screen(topics))
            .child(BoxView::with_full_screen(topic_info));

    cursive.add_layer(root_layout);

    cursive.run();
}

fn topics_select_callback(s: &mut Cursive, topic_metadata: &TopicMetadata) {
    s.call_on_id("topic_info", |topic_info: &mut TextView| {
       topic_info.set_content(format!("{:?}", topic_metadata))
    });
}

