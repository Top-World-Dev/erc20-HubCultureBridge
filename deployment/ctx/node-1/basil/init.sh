#!/bin/bash

set -e

source basil.env

PROJECT_NAME=${BASIL_PROJECT_NAME:?"Expecting BASIL_PROJECT_NAME in basil.env"}

shopt -s nullglob # set `nullglob` to prevent iteration on empty match

for file in ./accounts/*.env
do
    ( # use subshell to sanitize env vars

    # extract variables from file
    source "$file"

    ACCOUNT_NAME=${BASIL_ACCOUNT_NAME:?"Expecing BASIL_ACCOUNT_NAME in $file"}
    ACCOUNT_PASS=${BASIL_ACCOUNT_PASS:?"Expecting BASIL_ACCOUNT_PASS in $file"}
    ACCOUNT_SECRET=${BASIL_ACCOUNT_SECRET:?"Expecting BASIL_ACCOUNT_SECRET in $file"}
    ACCOUNT_ADDR=${BASIL_ACCOUNT_ADDR:?"Expecting BASIL_ACCOUNT_ADDR in $file"}

    PASSWORD_FILE="${ACCOUNT_NAME}.pass"

    # set up password file
    echo $ACCOUNT_PASS > $PASSWORD_FILE

    # insert secret into store...
    ethstore insert $ACCOUNT_SECRET  $PASSWORD_FILE --dir keys/$PROJECT_NAME

    # ensure that secret decrypts..
    ethstore sign $ACCOUNT_ADDR $PASSWORD_FILE c82a3ca1f9436de9ffe54faf3fef7e7ac76897e02ba7fd5d013b840fd350d01b --dir keys/$PROJECT_NAME
    
    ) # end subshell
done

shopt -u nullglob # unset `nullglob` (not typically expected behavior


echo "OK"

