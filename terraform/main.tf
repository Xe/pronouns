terraform {
  backend "s3" {
    bucket = "within-tf-state"
    key    = "fly-svc/pronouns"
    region = "us-east-1"
  }
}

data "aws_route53_zone" "dns" {
  name = "within.lgbt."
}

resource "aws_route53_record" "pronouns_challenge_CNAME" {
  zone_id = data.aws_route53_zone.dns.zone_id
  name    = "_acme-challenge.pronouns.within.lgbt."
  type    = "CNAME"
  records = ["pronouns.within.lgbt.qxdl2p.flydns.net."]
  ttl     = 300
}

resource "aws_route53_record" "pronouns_A" {
  zone_id = data.aws_route53_zone.dns.zone_id
  name    = "pronouns.within.lgbt."
  type    = "A"
  records = ["66.241.125.29"]
  ttl     = 300
}

resource "aws_route53_record" "pronouns_AAAA" {
  zone_id = data.aws_route53_zone.dns.zone_id
  name    = "pronouns.within.lgbt."
  type    = "AAAA"
  records = ["2a09:8280:1::a:c347"]
  ttl     = 300
}
