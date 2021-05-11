#!/bin/bash

set -e

set -u

TESTNET_DIR="testnet"

cd $TESTNET_DIR && \
    docker-compose down --volumes --remove-orphans && \
    cd ..

docker system prune --force

