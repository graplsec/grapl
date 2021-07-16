#!/usr/bin/env bash

# TODO: Deduplicate - https://github.com/grapl-security/issue-tracker/issues/614

# Build our AMI in Buildkite, supplying the necessary
# information to create metadata tags.
# This is called from .buildkite/pipeline.merge.ami-build.yml

set -euo pipefail

source .buildkite/scripts/lib/aws.sh
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

    # Both defined in packer.sh
    build_ami "${PACKER_IMAGE_NAME}"

    # HACK; see documentation of this function for details
    unset_aws_variables

    upload_manifest "${PACKER_IMAGE_NAME}"
}

build_packer_ci
