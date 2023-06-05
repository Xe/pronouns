terraform {
  backend "s3" {
    bucket = "within-tf-state"
    key    = "fly-svc/pronouns"
    region = "us-east-1"
  }
}

data "dns_a_record_set" "app" {
  host = "xe-pronouns.fly.dev"
}

data "dns_aaaa_record_set" "app" {
  host = "xe-pronouns.fly.dev"
}

data "aws_route53_zone" "dns" {
  name = "within.lgbt."
}

resource "aws_route53_record" "pronouns_A" {
  zone_id = data.aws_route53_zone.dns.zone_id
  name    = "pronouns.within.lgbt."
  type    = "A"
  records = data.dns_a_record_set.app.addrs
  ttl     = 300
}

resource "aws_route53_record" "pronouns_AAAA" {
  zone_id = data.aws_route53_zone.dns.zone_id
  name    = "pronouns.within.lgbt."
  type    = "AAAA"
  records = data.dns_aaaa_record_set.app.addrs
  ttl     = 300
}
