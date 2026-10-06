#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
version="$(bash "$script_dir/../ci/read-cargo-workspace-version.sh" --stable Cargo.toml)" || exit 1
commit="$(git rev-parse --verify HEAD)"
exec bash "$script_dir/../ci/check-release-tag.sh" "$commit" "$version"
