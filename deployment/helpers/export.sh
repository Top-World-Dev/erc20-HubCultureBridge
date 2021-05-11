#!/bin/bash

set -e

set -u

TARGET_DIR=${1:?'Target Directory Name'}

EXPORT_DIR="export"

OUTPUT_DIR=$EXPORT_DIR/$TARGET_DIR

mkdir -p $EXPORT_DIR

cp -rL $TARGET_DIR $OUTPUT_DIR

cp helpers/setup.sh $OUTPUT_DIR/setup.sh

cp helpers/prod.env $OUTPUT_DIR/.env

cp helpers/README.md $OUTPUT_DIR/README.md

cd $EXPORT_DIR && \
    zip -r $TARGET_DIR.zip $TARGET_DIR && \
    rm -r $TARGET_DIR
