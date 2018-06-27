#!/bin/bash

echo "Writing to topic \"$1\""

docker exec topiks_broker_1 \
    sh -c "echo $2 | kafka-console-producer --broker-list localhost:9092 --topic $1"
