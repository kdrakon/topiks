# topiks

An interactive CLI tool for managing Kafka topics.

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
```bash
topiks [-m] [-d] --bootstrap-server localhost:9092
```
- `-m` to enable topic modification
- `-d` to enable topic deletion

