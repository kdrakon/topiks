# topiks

[![Build Status](https://travis-ci.org/kdrakon/topiks.svg?branch=master)](https://travis-ci.org/kdrakon/topiks)

An interactive CLI tool for managing Kafka topics.

![screen capture](cursive.gif)

## About
Much of my time with Kafka clusters involves working directly on either brokers and/or other components (e.g. Kafka Connect, Schema Registry, etc.). There are a number of useful functions that Apache Kafka and Confluent have provided, but a lot of the time, what I desired was a single tool to perform menial topic tasks. Presently, I know there are plans for an in-house CLI tool to be built, but I thought I'd take a crack at one in the meantime.

## Features
- list topics, configurations, and offsets
- _interactively create topics (**WIP**)_
- selectively delete topics
- modify a topics configuration, replication factor
- _increase the partitions for a topic (**WIP**)_
- get offset and lag for a consumer group 

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

