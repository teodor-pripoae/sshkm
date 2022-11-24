#!/bin/bash

set -ex

# Build the builder docker image

sudo nerdctl build -t sshkm:builder -f Dockerfile.builder .

sudo nerdctl run --rm -v $(pwd)/..:/app sshkm:builder