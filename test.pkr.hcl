variable "aws_profile" {
  description = "The AWS connection profile to use when creating the image"
  type        = string
  default     = env("AWS_PROFILE")
}

variable "buildkite_step_key" {
  description = "The key of the step in the Buildkite pipeline this AMI was built in. If present, will be used to tag the instance used to build the AMI."
  type        = string
  default     = env("BUILDKITE_STEP_KEY")
}

locals {
  # We append a timestamp to the AMI name to create unique names.
  formatted_timestamp = formatdate("YYYYMMDDhhmmss", timestamp())
}

source "amazon-ebs" "testing" {
  ami_description = "Grapl Buildkite Base Image"
  ami_name        = "grapl-packer-test-${var.buildkite_step_key}-${local.formatted_timestamp}"
  instance_type   = "t2.micro"
  region          = "us-east-1"
  source_ami_filter {
    filters = {
      name                = "amzn2-ami-minimal-hvm-*"
      root-device-type    = "ebs"
      virtualization-type = "hvm"
      architecture        = "x86_64"
    }
    most_recent = true
    owners      = ["amazon"]
  }

  ssh_username    = "ec2-user"
  profile         = "${var.aws_profile}"

  tag {
    key = "Testing"
    value = "You Bet"
  }
}

build {
  sources = ["source.amazon-ebs.testing"]

  post-processor "manifest" {
    output = "packer-manifest.json" # The default value; just being explicit
  }

}
