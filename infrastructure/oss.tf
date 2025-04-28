resource "random_integer" "default" {
  max = 99999
  min = 10000
}

resource "alicloud_oss_bucket" "bucket-website" {
  bucket = "imaginarium-${random_integer.default.result}"
  website {
    index_document = "index.html"
  }
}

resource "alicloud_oss_bucket_acl" "default" {
  bucket = alicloud_oss_bucket.bucket-website.bucket
  acl = "public-read"
}

resource "alicloud_oss_bucket_https_config" "default" {
  tls_versions = ["TLSv1.3", "TLSv1.2"]
  bucket = alicloud_oss_bucket.bucket-website.bucket
  enable = true
}