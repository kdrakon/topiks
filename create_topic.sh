#!/bin/bash

echo "Creating topic \"$1\""

docker exec topiks_broker_1 \
    kafka-topics --zookeeper zookeeper:2181 --create --if-not-exists --topic $1 --partitions 16 --replication-factor 1
