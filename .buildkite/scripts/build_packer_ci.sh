#!/usr/bin/env bash

# TODO: Deduplicate - https://github.com/grapl-security/issue-tracker/issues/614

# Build our AMI in Buildkite, supplying the necessary
# information to create metadata tags.
# This is called from .buildkite/pipeline.merge.ami-build.yml

set -euo pipefail

source .buildkite/scripts/lib/packer.sh

export GIT_SHA="${BUILDKITE_COMMIT}"
export GIT_BRANCH="${BUILDKITE_BRANCH}"
# This : syntax does nothing; but in unison with `set -u` marks these two vars as required.
: "${BUILDKITE_BUILD_NUMBER}"
# it is worried that I confused this with PACKER_IMAGE_NAMES. I didn't.
# shellcheck disable=SC2153
: "${PACKER_IMAGE_NAME}"

build_packer_ci() {
    echo -e "--- :packer: Performing build of AMI"

    echo "--- Identity test before Packer run"
    aws sts get-caller-identity

    # Both defined in packer.sh
    build_ami "${PACKER_IMAGE_NAME}"

    echo "--- Unsetting AWS environment variables set by aws-assume-role plugin"
    unset AWS_ACCESS_KEY_ID
    unset AWS_SECRET_ACCESS_KEY
    unset AWS_SESSION_TOKEN

    echo "--- Identity test after Packer run"
    aws sts get-caller-identity

    upload_manifest "${PACKER_IMAGE_NAME}"
}

build_packer_ci
