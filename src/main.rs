extern crate byteorder;
extern crate cursive;
#[macro_use]
extern crate proptest;

use cursive::align::*;
use cursive::Cursive;
use cursive::theme::*;
use cursive::traits::*;
use cursive::views::*;
use kafka_protocol::protocol_request::*;
use kafka_protocol::protocol_requests::metadata_request::*;
use kafka_protocol::protocol_response::*;
use kafka_protocol::protocol_responses::metadata_response::*;
use kafka_protocol::protocol_serializable::*;

pub mod kafka_protocol;
pub mod tcp_stream_util;

fn main() {
    let metadata_request = MetadataRequest { topics: None, allow_auto_topic_creation: false };

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

    let response: Response<MetadataResponse> =
        tcp_stream_util::request("localhost:9092", request, |bytes| { bytes.into_protocol_type() }).unwrap();

    let mut cursive = Cursive::new();
    cursive.set_theme(Theme { shadow: false, borders: BorderStyle::Simple, palette: default_palette() });
    cursive.add_global_callback('q', |s| s.quit());

    let brokers =
        broker_metadata_textview(response.response_message.cluster_id, response.response_message.controller_id, response.response_message.brokers)
            .align(Align::new(HAlign::Right, VAlign::Center));

    let topics =
        topic_metadata_selectview(response.response_message.topic_metadata);

    let mut topic_info = StackView::new();
    topic_info.add_fullscreen_layer(BoxView::with_full_screen(TextView::new("Topiks")));

    let root_layout =
        LinearLayout::vertical()
            .child(Panel::new(brokers))
            .child(LinearLayout::horizontal()
                .child(BoxView::with_full_screen(topics))
                .child(Panel::new(topic_info.with_id("topic_info")))
            );

    cursive.add_fullscreen_layer(root_layout);

    cursive.run();
}

fn broker_metadata_textview(cluster_id: Option<String>, controller_id: i32, brokers: Vec<BrokerMetadata>) -> TextView {
    let cluster =
        format!("Cluster: {} Controller ID: {}\n", cluster_id.unwrap_or(String::from("n/a")), controller_id);

    let brokers =
        brokers.into_iter().map(|metadata| {
            format!(" {} [{}:{} rack:{}]", metadata.node_id, metadata.host, metadata.port, metadata.rack.unwrap_or(String::from("None")))
        }).fold(String::new(), |acc, s| {
            format!("{}{}", acc, s)
        });

    TextView::new(format!("{} {}", cluster, brokers))
}

fn topic_metadata_selectview(topic_metadatas: Vec<TopicMetadata>) -> SelectView<TopicMetadata> {
    let mut topics = SelectView::<TopicMetadata>::new().on_select(topics_select_callback);

    topic_metadatas.into_iter().for_each(|topic_metadata| {
        topics.add_item(topic_metadata.topic.clone(), topic_metadata)
    });

    topics
}

fn topics_select_callback(s: &mut Cursive, topic_metadata: &TopicMetadata) {
    let topic_header =
        TextView::new(format!("Topic: {} | Partitions: {} | Internal: {}", topic_metadata.topic.clone(), topic_metadata.partition_metadata.len(), topic_metadata.is_internal));

    let mut partitions = topic_metadata.partition_metadata.clone();
    partitions.sort_by_key(|e| e.partition);

    let header = format!("{:9} | {:9} | {:9} | {:9} | {:9}", "partition", "leader", "replicas", "isr", "offline replicas");
    let partition_data = TextView::new(
        partitions.iter().map(|partition| {
            let vec_to_string: fn(String, &i32) -> String = |acc, r| { format!("{} {}", acc, r) };
            let replicas = partition.replicas.iter().fold(String::new(), vec_to_string);
            let isr = partition.isr.iter().fold(String::new(), vec_to_string);
            let offline_replicas = partition.offline_replicas.iter().fold(String::new(), vec_to_string);
            format!("{:9} | {:9} | {:9} | {:9} | {:9}", partition.partition, partition.leader, replicas, isr, offline_replicas)
        }).fold(header, |lines, line| {
            format!("{}\n{}", lines, line)
        })
    );

    let topic_metadata_layout =
        LinearLayout::vertical()
            .child(Panel::new(topic_header))
            .child(partition_data);

    s.call_on_id("topic_info", |topic_info: &mut StackView| {
        topic_info.pop_layer();
        topic_info.add_fullscreen_layer(BoxView::with_full_screen(topic_metadata_layout));
    });
}

