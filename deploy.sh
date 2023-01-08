#!/usr/bin/env bash

set -ex

nix build .#docker
docker load < ./result
docker push registry.fly.io/xe-pronouns:latest
flyctl deploy
