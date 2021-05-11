#!/bin/bash

# halt on error
set -e

# halt on undefined
set -u

# Ensure apt caches are up to date
apt-get update

# --------- Docker CE Install ---------

# Install docker dependencies
apt-get install -y \
    apt-transport-https \
    ca-certificates \
    curl \
    software-properties-common

# Add docker GPG key
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | apt-key add -

# show expected fingerprint
echo "Expect Fingerprint: 9DC8 5822 9FC7 DD38 854A E2D8 8D81 803C 0EBF CD88"

# show fingerprint
apt-key fingerprint 0EBFCD88

# Add docker respository
add-apt-repository -y \
    "deb [arch=amd64] https://download.docker.com/linux/ubuntu \
    $(lsb_release -cs) \
    stable"

# Re-update chaches now that the docker repository has been added
apt-get update

# Install Docker CE
apt-get install -y docker-ce

# --------- Docker Compose Install ---------

# Install pip package-manager
apt-get install -y python3-pip

# Install Docker Compose
pip3 install docker-compose

