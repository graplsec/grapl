#!/usr/bin/env bash

set -euo pipefail

source .buildkite/scripts/lib/artifacts.sh
source .buildkite/scripts/lib/cloudsmith.sh
source .buildkite/scripts/lib/version.sh

main() {

    # This is the name defined in the top-level Makefile in the
    # `ux-tarball` target.
    local -r UX_FILENAME="grapl-ux.tar.gz"
    local -r UX_ARTIFACTS_FILE="ux_artifacts.json"

    echo "--- :buildkite: Retrieving UX tarball"
    buildkite-agent artifact download "dist/${UX_FILENAME}" .

    local -r version=$(timestamp_and_sha_version)

    echo "--- :cloudsmith: Uploading ${UX_FILENAME} version ${version}"
    upload_raw_file_to_cloudsmith "dist/${UX_FILENAME}" "${version}" raw

    artifact_json "${version}" "grapl-ux" > "${UX_ARTIFACTS_FILE}"

    echo "--- :buildkite: Uploading ${UX_ARTIFACTS_FILE} file"
    buildkite-agent artifact upload "${UX_ARTIFACTS_FILE}"
}

main
