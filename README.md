# topiks

[![Build Status](https://travis-ci.org/kdrakon/topiks.svg?branch=master)](https://travis-ci.org/kdrakon/topiks)
[![Release](https://img.shields.io/github/tag-date/kdrakon/topiks.svg?style=popout)](https://github.com/kdrakon/topiks/releases)

An interactive CLI tool for managing Kafka topics.

## Features
- compatible with Apache Kafka >=2.0
- list topics, configurations, and offsets
- interactively create topics
- selectively delete topics
- modify a topics configuration
- _modify a topics replication factor (**WIP**)_
- _increase the partitions for a topic (**WIP**)_
- get offset and lag for a consumer group 
- TLS/SSL protocol support

## Usage
```
USAGE:
    topiks [FLAGS] [OPTIONS] <bootstrap-server>

FLAGS:
    -D                              Enable topic/config deletion
    -h, --help                      Prints help information
    -M                              Enable creation of topics and modification of topic configurations
        --no-delete-confirmation    Disable delete confirmation <Danger!>
        --tls                       Enable TLS
    -V, --version                   Prints version information

OPTIONS:
    -c, --consumer-group <consumer-group>    Consumer group for fetching offsets

ARGS:
    <bootstrap-server>    A single Kafka broker [DOMAIN|IP]:PORT
```

### Commands
```
 h → Toggle this screen
 q → Quit
 t → Toggle topics view
 i → Toggle topic config view
 p → Toggle partitions view
 / → Enter search query for topic name
 n → Find next search result
 r → Refresh. Retrieves metadata from Kafka cluster
 c → Create a new topic with [topic]:[partitions]:[replication factor]
 : → Modify a resource (e.g. topic config) via text input
 d → Delete a resource. Will delete a topic or reset a topic config
 Up⬆ → Move up one topic
 Down⬇ → Move down one topic
 PgUp⇞ → Move up ten topics
 PgDown⇟ → Move down ten topics
 Home⤒ → Go to first topic
 End⤓ → Go to last topic
```

## Build your own
1. Install rust and Cargo: https://rustup.rs/
1. `RUST_TARGET=??? make build` where `RUST_TARGET` is the rust toolchain. For example, `x86_64-apple-darwin` or `x86_64-unknown-linux-gnu`
1. The binary will be in `target/${RUST_TARGET}/release/`

## FAQ

> What was the main reason for building this?

I found a substantial amount of my time developing Kafka applications—namely via Kafka Streams—involved keeping an eye on the consumer groups I was using. This primarily meant tracking the group offsets of the topics my applications were reading and writing. With a mix of `grep`, `awk`, and `sed` commands, we would do this by periodically reading data from `kafka-consumer-groups`. Although this worked, it wasn't ideal when we started working on other applications or had different topics and consumer groups we wanted to track. 

Furthermore, when developing new Kafka applications, we found ourselves periodically creating and deleting topics in our local and test environments.

For these reasons and more, this led me to imagining a Kafka topics tool that would be in the same vein as [`htop`](https://github.com/hishamhm/htop).

> Why can't you read or write to topics?

Although that feature wouldn't be completely difficult to implement, there are two reasons I prefer not to do so:
1. In almost all of my experience with Kafka, the data we read/write is in some schema format (e.g. Avro). This in-turn implies in most cases cumbersome work for the user to both correctly produce and consume via the command-line.
1. Topiks would have to work with its own consumer group or one specified by the user. I'd prefer to minimize the impact that Topiks leaves on your Kafka cluster, such as creating throwaway consumer groups. 

> How do you support TLS?

TLS/SSL is capable via the [rust-native-tls](https://github.com/sfackler/rust-native-tls) crate. This library provides bindings to native TLS implementations based on your operating system. For this reason, you may find the binaries I release not compatible with your OS. The simple solution is to then build Topiks on your machine.


## A Quick Look

- Listing and searching for topics

![Listing and searching for topics](https://media.giphy.com/media/9Pgz8yJOB8RDTOyDPL/source.gif)

- Looking at partitions and offsets. In this case, we can see the consumer group has read three messages across the partitions.

![Looking at partitions and offsets](https://media.giphy.com/media/fjyp0OZqs0TXIZbeil/source.gif)

- Topic configuration. In this case, Topiks applies a config override. To reset the override, simply _delete_ (`d`) the config and it will revert to the global default.

![Topic configuration](https://media.giphy.com/media/fQovPSOuIwB6BIle7q/source.gif)

- Topic creation and deletion.

![Topic creation and deletion](https://media.giphy.com/media/9oIFgyK3paNmmo1RUW/source.gif)

- You can also override the delete confirmation if you're absolutely sure you know what you're doing.

![Topic deletion with no confirmation](https://media.giphy.com/media/9D6PuYyr7rdXWUDozH/source.gif)

