#!/bin/bash
# author : Tharanga

if [ "`id -u`" != 0 ]; then
    echo "Please run as root!";
    exit 1;
fi

# get the user input to extract the source
read -p "What is the application type? : " APP
if [ ! -f "${APP}.zip" ]; then
    echo "No source found called '${APP}.zip'!"
    exit 1
fi

SERVICE_DIR="/srv/${APP}"
SERVICE_DEPLOYMENT_DIR="${SERVICE_DIR}/releases/`date +%Y%m%d%H%M%S`"

mkdir -p ${SERVICE_DEPLOYMENT_DIR}

unzip -qq ${APP}.zip -d ${SERVICE_DEPLOYMENT_DIR}

# update the symlink to point to the new code
rm -f ${SERVICE_DIR}/current
ln -s ${SERVICE_DEPLOYMENT_DIR} ${SERVICE_DIR}/current

ls -la ${SERVICE_DIR}
