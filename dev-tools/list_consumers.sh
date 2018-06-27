#!/bin/bash

docker exec topiks_broker_1 \
    kafka-consumer-groups --bootstrap-server localhost:9092 --list