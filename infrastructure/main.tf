terraform {
  required_providers {
    alicloud = {
      source = "aliyun/alicloud"
      version = "1.248.0"
    }
  }
}

provider "alicloud" {
  access_key = var.alicloud_access_key
  secret_key = var.alicloud_secret_key
  region = var.region
}

variable "instance_type" {
  default = "ecs.c8i.large"
}

variable "region" {
  default = "ap-southeast-1"
}

resource "alicloud_vpc" "vpc" {
  vpc_name = "imaginarium-vpc"
  cidr_block = "10.16.0.0/16"
}

resource "alicloud_key_pair" "key" {
  key_pair_name = "imaginarium-admin"
  public_key = file("./keys/admin.pub")
}

resource "alicloud_instance" "instance" {
  availability_zone = data.alicloud_zones.default.zones.0.id
  security_groups = alicloud_security_group.default.*.id
  key_name = alicloud_key_pair.key.key_name
  instance_type = var.instance_type
  system_disk_category = "cloud_essd"
  system_disk_size = 20
  system_disk_name = "imaginarium-disk"
  image_id = "ubuntu_24_04_x64_20G_alibase_20250317.vhd"
  instance_name = "imaginarium-001"
  vswitch_id = alicloud_vswitch.vswitch.id
  internet_max_bandwidth_out = 5
  host_name = "imaginarium"
}

resource "alicloud_cr_namespace" "registry" {
  name = "imaginarium"
  auto_create = false
  default_visibility = "PRIVATE"
}

resource "alicloud_cr_repo" "server-repository" {
  namespace = alicloud_cr_namespace.registry.name
  name = "server"
  repo_type = "PRIVATE"
  summary = "imaginarium server"
}

resource "alicloud_cdn_domain_new" "cdn" {
  scope = "overseas"
  domain_name = "imaginarium.monster"
  cdn_type = "web"
  sources {
    type = "oss"
    content = alicloud_oss_bucket.bucket-website.bucket
    priority = 20
    port = 443
  }
}

output "ip_address" {
  value = alicloud_instance.instance.public_ip
}
