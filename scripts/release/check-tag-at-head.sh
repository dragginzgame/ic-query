#!/usr/bin/env bash
set -euo pipefail

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: failed to read a release version from Cargo.toml" >&2
  exit 1
fi

if ! tag_commit="$(git rev-parse "v${version}^{}" 2>/dev/null)"; then
  echo "error: release tag v${version} does not exist" >&2
  exit 1
fi
head_commit="$(git rev-parse HEAD)"
if [[ "${tag_commit}" != "${head_commit}" ]]; then
  echo "error: release tag v${version} does not point to HEAD" >&2
  exit 1
fi
