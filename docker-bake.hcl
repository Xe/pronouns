variable "REGISTRY" {
  default = "ghcr.io/xe"
}

group "default" {
  targets = ["pronouns"]
}

target "pronouns" {
  dockerfile = "Dockerfile"
  tags = [
    "${REGISTRY}/pronouns:latest",
  ]
  platforms = ["linux/amd64"]
}
