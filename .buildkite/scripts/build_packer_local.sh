#!/usr/bin/env bash

# Build our AMI on a local workstation, supplying the necessary
# information to create metadata tags.

set -euo pipefail

source .buildkite/scripts/lib/packer.sh

GIT_SHA=$(git rev-parse --short HEAD)
export GIT_SHA
GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
export GIT_BRANCH

build_ami
