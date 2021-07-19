#!/bin/sh
# Adapted from:
# - https://vector.dev/docs/setup/quickstart/
# - https://vector.dev/docs/setup/installation/package-managers/apt/
set -euo pipefail

curl -1sLf \
  'https://repositories.timber.io/public/vector/cfg/setup/bash.deb.sh' \
| sudo -E bash

sudo apt-get install "vector=${VECTOR_VERSION}"

# Ensure it's all working correctly
vector --version