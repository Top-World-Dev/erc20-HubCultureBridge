#!/bin/bash

set -e

set -u

TESTNET_DIR="testnet"

LOGSERVER_DIR="logserver"

AUTHORITY_DIR="authority"

VAULT_DIR="vault"


cd $TESTNET_DIR && \
    docker-compose up -d --build && \
    cd ..

cd $LOGSERVER_DIR && \
    docker-compose up -d --build && \
    cd ..

cd $AUTHORITY_DIR && \
    docker-compose up -d --build && \
    cd ..

cd $VAULT_DIR && \
    docker-compose up -d --build && \
    cd ..
