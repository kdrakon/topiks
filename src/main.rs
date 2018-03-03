use kafka_protocol::*;
use kafka_protocol::protocol_primitives::*;
use kafka_protocol::protocol_serializable::*;
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
                let mut buffer: [u8; 1024] = [0; 1024];
                stream.read(&mut buffer).map(|result_size| (buffer, result_size))
            })
        });

    match send_result {
        Ok((buffer, size)) => {
            println!("printing results...");
            let (result, _) = buffer.split_at(size);
            for b in result.iter() {
                print!("{:02X} ", b)
            }
        }
        Err(e) => println!("failed, {:?}", e)
    };
}

