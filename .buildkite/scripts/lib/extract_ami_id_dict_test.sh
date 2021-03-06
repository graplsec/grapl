#!/usr/bin/env bash

readonly jq_filter_path=".buildkite/scripts/lib/extract_ami_id_dict.jq"
readonly example_manifest_file=".buildkite/scripts/lib/example_packer_artifact.json"

test_extract_ami_id_dict() {
    packer_image_name="imgname"
    actual=$(jq --raw-output --arg IMAGE_NAME ${packer_image_name} --from-file "${jq_filter_path}" "${example_manifest_file}")
    expected=$(
        cat << EOF
{
  "imgname.us-east-1": "ami-111",
  "imgname.us-east-2": "ami-222",
  "imgname.us-west-1": "ami-333",
  "imgname.us-west-2": "ami-444"
}
EOF
    )

    assertEquals "Failed to generate expected JSON" \
        "${expected}" \
        "${actual}"
}
