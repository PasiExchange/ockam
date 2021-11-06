#!/usr/bin/env bash

val=$(eval "cargo metadata --no-deps | jq '[.packages[] | {name: .name, version: .version, release: .metadata.release.release}]'")
length=$(eval "echo '$val' | jq '. | length' ")
echo "$length"