extern crate byteorder;

use kafka_protocol::protocol_request::*;
use kafka_protocol::protocol_primitives::*;
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

    match send_result {
        Ok(result) => {
            println!("printing results...");
            for b in result.iter() {
                print!("{:02X} ", b)
            }
        }
        Err(e) => println!("failed, {:?}", e)
    };
}

