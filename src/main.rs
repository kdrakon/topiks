extern crate byteorder;
extern crate cursive;
#[macro_use]
extern crate proptest;

use cursive::align::*;
use cursive::Cursive;
use cursive::event::*;
use cursive::theme::*;
use cursive::traits::*;
use cursive::view::Identifiable;
use cursive::views::*;
use kafka_protocol::protocol_request::*;
use kafka_protocol::protocol_requests::metadata_request::*;
use kafka_protocol::protocol_response::*;
use kafka_protocol::protocol_responses::metadata_response::*;
use kafka_protocol::protocol_serializable::*;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub mod kafka_protocol;
pub mod tcp_stream_util;
pub mod main_channel;

fn main() {
    let metadata_request = MetadataRequest { topics: None, allow_auto_topic_creation: false };

    let response: Response<MetadataResponse> =
        tcp_stream_util::request("localhost:9092", metadata_request.into_v5_request(), |bytes| { bytes.into_protocol_type() }).unwrap();

    let mut shared_cursive = Arc::new(Mutex::new(Cursive::new()));
    let sender = main_channel::of(shared_cursive.clone());
    let mut cursive = shared_cursive.lock().unwrap();

    cursive.set_theme(Theme { shadow: false, borders: BorderStyle::Simple, palette: default_palette() });
    cursive.add_global_callback('q', |c| c.quit());

    let brokers =
        broker_metadata_textview(response.response_message.cluster_id, response.response_message.controller_id, response.response_message.brokers)
            .align(Align::new(HAlign::Right, VAlign::Center));

    let topics =
        OnEventView::new(topic_metadata_selectview(response.response_message.topic_metadata).with_id("topics"))
            .on_pre_event('d', delete_topic_callback);

    let mut info_view = StackView::new();
    info_view.add_fullscreen_layer(BoxView::with_full_screen(TextView::new("Topiks")));

    let root_layout =
        LinearLayout::vertical()
            .child(Panel::new(brokers))
            .child(LinearLayout::horizontal()
                .child(BoxView::with_full_screen(topics))
                .child(Panel::new(info_view.with_id("info_view")))
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

fn topics_select_callback(cursive: &mut Cursive, topic_metadata: &TopicMetadata) {
    let topic_header =
        TextView::new(format!("Topic: {}\nPartitions: {} | Internal: {}", topic_metadata.topic.clone(), topic_metadata.partition_metadata.len(), topic_metadata.is_internal));

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

    cursive.call_on_id("info_view", |topic_info: &mut StackView| {
        topic_info.pop_layer();
        topic_info.add_fullscreen_layer(BoxView::with_full_screen(topic_metadata_layout));
    });
}

fn delete_topic_callback(cursive: &mut Cursive) {
    let delete_dialog =
        cursive.call_on_id("topics", |topics: &mut SelectView<TopicMetadata>| {
            topics.selected_id().and_then(|id| {
                topics.get_item(id).map(|(topic, topic_metadata)| {
                    Dialog::around(
                        LinearLayout::vertical()
                            .child(TextView::new("Delete Topic? Complete the name:"))
                            .child(delete_verification_view(topic)))
                        .title("Delete Topic")
                        .button("Cancel", |cursive| cursive.pop_layer())
                })
            })
        });

    if let Some(Some(dialog)) = delete_dialog {
        cursive.add_layer(dialog);
    }
}

fn delete_verification_view(topic: &str) -> LinearLayout {
    let (head, tail) =
        match topic.len() {
            len if len <= 3 => ("", topic),
            len => (&topic[0..len - 3], &topic[len - 3..])
        };

    let verification = move |accept: String| {
        OnEventView::new(TextArea::new()).on_pre_event_inner(Key::Enter, move |textarea| {
            if textarea.get_content().eq(accept.as_str()) {
                println!("deleting");
                Some(EventResult::Consumed(None))
            } else {
                textarea.set_content("");
                Some(EventResult::Consumed(None))
            }
        })
    };

    LinearLayout::horizontal()
        .child(TextView::new(head))
        .child(verification(String::from(tail)))
}

