resource "alicloud_security_group" "default" {
  security_group_name = "imaginarium-security-group"
  description = "foo"
  vpc_id = alicloud_vpc.vpc.id
}

data "alicloud_zones" "default" {
  available_disk_category = "cloud_efficiency"
  available_resource_creation = "VSwitch"
  available_instance_type = var.instance_type
}

resource "alicloud_vswitch" "vswitch" {
  vpc_id = alicloud_vpc.vpc.id
  cidr_block = "10.16.0.0/24"
  zone_id = data.alicloud_zones.default.zones.0.id
  vswitch_name = "imaginarium-switch"
}

resource "alicloud_security_group_rule" "http" {
  type = "ingress"
  ip_protocol = "tcp"
  nic_type = "intranet"
  policy = "accept"
  priority = 1
  port_range = "80/80"
  security_group_id = alicloud_security_group.default.id
  cidr_ip = "0.0.0.0/0"
}

resource "alicloud_security_group_rule" "https" {
  type = "ingress"
  ip_protocol = "tcp"
  nic_type = "intranet"
  policy = "accept"
  priority = 1
  port_range = "443/443"
  security_group_id = alicloud_security_group.default.id
  cidr_ip = "0.0.0.0/0"
}

resource "alicloud_security_group_rule" "ssh" {
  type = "ingress"
  ip_protocol = "tcp"
  nic_type = "intranet"
  policy = "accept"
  priority = 1
  port_range = "22/22"
  security_group_id = alicloud_security_group.default.id
  cidr_ip = "0.0.0.0/0"
}
