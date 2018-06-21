# topiks

[![Build Status](https://travis-ci.org/kdrakon/topiks.svg?branch=master)](https://travis-ci.org/kdrakon/topiks)

An interactive CLI tool for managing Kafka topics.

![screen capture](cursive.gif)

## About
Much of my time with Kafka clusters involves working directly on either brokers and/or other components (e.g. Kafka Connect, Schema Registry, etc.). There are a number of useful functions that Apache Kafka and Confluent have provided, but a lot of the time, what I desired was a single tool to perform menial topic tasks. Presently, I know there are plans for an in-house CLI tool to be built, but I thought I'd take a crack at one in the meantime.

## Features
- list topics, configurations, and offsets
- interactively create topics
- selectively delete topics
- modify a topics configuration, replication factor
- increase the partitions for a topic
- get offset and lag for a consumer group 

## Usage
```
USAGE:
    topiks [FLAGS] [OPTIONS] <bootstrap-server>

FLAGS:
    -D                              Enable topic deletion
    -h, --help                      Prints help information
        --no-delete-confirmation    Disable delete confirmation <Danger!>
    -V, --version                   Prints version information

OPTIONS:
    -c, --consumer-group <consumer-group>    Consumer group for fetching offsets

ARGS:
    <bootstrap-server>    A single Kafka broker to connect to
```

