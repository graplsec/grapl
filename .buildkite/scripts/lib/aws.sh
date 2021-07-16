#!/usr/bin/env bash

# Unsets AWS credential environment variables. This can be used to
# e.g., drop a previously assumed IAM role and fall back to an EC2
# instance role.
#
# NOTE: This is essentially a hack until we work around some issues in
# our Buildkite + AWS environment, and how some of our jobs are
# structured. Unless you are *absolutely certain* you need to use this
# function, don't use this function.
#
# Please see
# https://github.com/grapl-security/issue-tracker/issues/620 for
# additional context.
unset_aws_variables() {
    echo -e "--- :aws: Unsetting AWS credential environment variables"
    _unset_and_log AWS_ACCESS_KEY_ID
    _unset_and_log AWS_SECRET_ACCESS_KEY
    _unset_and_log AWS_SESSION_TOKEN
}

# Just logs when we unset a variable, for better visibility in our
# Buildkite logs.
_unset_and_log() {
    local -r variable="${1}"

    echo "Unsetting ${variable} environment variable"
    unset "${variable}"
}
