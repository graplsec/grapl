variable "aws_profile" {
  description = "The AWS connection profile to use when creating the image"
  type        = string
  default     = env("AWS_PROFILE")
}

source "amazon-ebs" "testing" {
  ami_description = "Grapl Buildkite Base Image"
  ami_name        = "grapl-packer-test"
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
