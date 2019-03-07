# topiks

[![Build Status](https://travis-ci.org/kdrakon/topiks.svg?branch=master)](https://travis-ci.org/kdrakon/topiks)
[![Release](https://img.shields.io/github/tag-date/kdrakon/topiks.svg?style=popout)](https://github.com/kdrakon/topiks/releases)

An interactive CLI tool for managing Kafka topics.

## Features
- compatible with Apache Kafka >=2.0
- list topics, configurations, and offsets
- _interactively create topics (**WIP**)_
- selectively delete topics
- modify a topics configuration
- _modify a topics replication factor (**WIP**)_
- _increase the partitions for a topic (**WIP**)_
- get offset and lag for a consumer group 
- TLS/SSL protocol support

## A Quick Look

![Listing and searching for topics](gifs/list-and-search.gif)
- Listing and searching for topics

![Looking at partitions and offsets](gifs/partitions-and-offsets.gif)
- Looking at partitions and offsets

## Usage
```
USAGE:
    topiks [FLAGS] [OPTIONS] <bootstrap-server>

FLAGS:
    -D                              Enable topic/config deletion
    -h, --help                      Prints help information
    -M                              Enable modification of topic configurations and other resources
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
 p → Toggle partitions
 i → Toggle topic config
 / → Enter search query for topic name
 n → Find next search result
 r → Refresh. Retrieves metadata from Kafka cluster
 : → Modify a resource (e.g. topic config) via text input
 d → Delete a resource. Will delete a topic or reset a topic config
 Up⬆ → Move up one topic
 Down⬇ → Move down one topic
 PgUp⇞ → Move up ten topics
 PgDown⇟ → Move down ten topics
 Home⤒ → Go to first topic
 End⤓ → Go to last topic
```

## FAQ

> What was the main reason for building this?

I found a lot of my time developing Kafka applications—namely Kafka Streams—involved keeping an eye on the consumer groups I was using. This primarily meant tracking the group offsets of the topics my applications were reading and writing. With a mix of `grep`, `awk`, and `sed` commands, we would periodically read data from `kafka-consumer-groups`. Although this worked, it wasn't ideal when we moved to other applications or had different topics and consumer groups we wanted to track. This led me to imagining a Kafka topics tool that would be in the same vain

> Why can't you read or write to topics?

Although that feature wouldn't be overly difficult to implement, there are two reasons I prefer not to do so:
1. In almost all of my experience with Kafka, the data we read/write is in some schema format (e.g. Avro). This in-turn means there would be some cumbersome work for both producing and consuming via the command-line. Ideally, if you really want to read/write from there, you can use the bundled CLI tools from Kafka and Confluent.
1. Topiks would have to work with a consumer group
