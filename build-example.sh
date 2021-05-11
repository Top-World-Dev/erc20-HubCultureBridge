#!/bin/bash

set -e

set -u

PROJECT_DIR="example"

OUTPUT_DIR="tmp/output"

CONTRACT_DIR="HubCultureSolidity/contracts"

PROXY_DIR="HubCultureProxy"

MIDDLEWARE_CTX="middleware"

LOGS_START_BLOCK="0x0"

LOGS_CALLBACK="stdout"

# populate latest contract source

cp -r $CONTRACT_DIR/* $PROJECT_DIR/config/contracts/

# trigger primary build

cd $PROJECT_DIR

basil --log debug build --output ../$OUTPUT_DIR

cd ..

# populate docker build contexts

cp -r $PROXY_DIR/vault-signer $OUTPUT_DIR/vault-signer

#cp $PROJECT_DIR/test-keys/vault.key $OUTPUT_DIR/vault-signer/secret.key

cp -r $PROXY_DIR/authority-signer $OUTPUT_DIR/authority-signer

cp -r $PROXY_DIR/admin-signer $OUTPUT_DIR/owner-signer

cp $PROJECT_DIR/test-keys/owner.key $OUTPUT_DIR/owner-signer/secret.key

cp -r $PROXY_DIR/user-signer $OUTPUT_DIR/user-signer

cp $PROJECT_DIR/test-keys/user.key $OUTPUT_DIR/user-signer/secret.key

#cp $PROJECT_DIR/test-keys/authority.key $OUTPUT_DIR/authority-signer/secret.key

cp -r $PROXY_DIR/log-server $OUTPUT_DIR/log-server

cp -r $MIDDLEWARE_CTX $OUTPUT_DIR/$MIDDLEWARE_CTX

# pupulate required environment variables

source $OUTPUT_DIR/interface/basil/basil.env

echo 'HC_NODE_WS=ws://'$BASIL_NETWORK_HOST':8546' >> $OUTPUT_DIR/.env

echo 'HC_CONTRACT=0x'$BASIL_CONTRACT_HUBCULTURE >> $OUTPUT_DIR/.env

echo 'LOGS_START_BLOCK='$LOGS_START_BLOCK >> $OUTPUT_DIR/.env

echo 'LOGS_CALLBACK='$LOGS_CALLBACK >> $OUTPUT_DIR/.env
