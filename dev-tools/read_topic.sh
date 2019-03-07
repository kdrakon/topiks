#!/bin/bash

echo "Reading topic \"$1\""

docker exec topiks_broker_1 \
    kafka-console-consumer --bootstrap-server localhost:9092 --from-beginning --topic $1 --max-messages $2 --group $3
