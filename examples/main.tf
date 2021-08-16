terraform {
  required_providers {
    aws = "~> 3"
  }
  required_version = "= 0.14.7"
}

provider "aws" {
  region = "eu-west-1"
}

resource "aws_s3_bucket" "website" {
  bucket = "my-unique-bucket"
  acl    = "public-read"
  website {
    index_document = "index.html"
  }
  tags = {
    kind = "demo"
  }
}

locals {
  title = "Demo"
}

variable "greeting" {
  type    = string
  default = "How are y'all doing?"
}

resource "aws_s3_bucket_object" "object" {
  bucket       = aws_s3_bucket.website.id
  key          = "index.html"
  content_type = "text/html"
  content      = <<EOF
<!doctype html>
<html>
  <head>
    <meta charset="utf-8"/>
    <title>${local.title}</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/water.css">
  </head>
  <body>
    <h1>Hello!</h1>
    <p>How are y'all doing? 👋<p>
  </body>
</html>
  EOF
}

resource "aws_s3_bucket_policy" "can_be_read" {
  bucket = aws_s3_bucket.website.id
  policy = data.aws_iam_policy_document.public_can_read.json
}

data "aws_iam_policy_document" "public_can_read" {
  statement {
    sid    = "PublicReadGetObject"
    effect = "Allow"
    principals {
      type        = "AWS"
      identifiers = ["*"]
    }
    actions = [
      "s3:GetObject"
    ]
    resources = [
      "arn:aws:s3:::${aws_s3_bucket.website.id}",
      "arn:aws:s3:::${aws_s3_bucket.website.id}/*"
    ]
  }
}

output "url" {
  value = "https://${aws_s3_bucket.website.bucket_regional_domain_name}/index.html"
}
