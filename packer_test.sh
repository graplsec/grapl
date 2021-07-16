#!/usr/bin/env bash


dropping_steps=(
    with-drop
)

uploading_steps=(
    no-drop
    with-drop
)

should_drop() {
    for step in "${dropping_steps[@]}"; do
        if [ "${BUILDKITE_STEP_KEY}" == "${step}" ]; then
            return 0
        fi
    done
    return 1
}

should_upload() {
    for step in "${uploading_steps[@]}"; do
        if [ "${BUILDKITE_STEP_KEY}" == "${step}" ]; then
            return 0
        fi
    done
    return 1
}

drop_role() {
    export AWS_ACCESS_KEY_ID
    export AWS_SECRET_ACCESS_KEY
    export AWS_SESSION_TOKEN
}

echo "--- :aws: Identity Before Packer"
aws sts get-current-identity

echo "--- :packer: Packer build"
packer build test.pkr.hcl

if should_drop; then
    echo "--- Dropping role"
    drop_role
fi

if should_upload; then
    echo "--- :aws: Identity before Upload (in command)"
    aws sts get-current-identity

    echo "--- :buildkite: Uploading manifest"
    buildkite-agent artifact upload packer-manifest.json
fi
