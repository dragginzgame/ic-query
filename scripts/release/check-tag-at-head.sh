#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
version="$(CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash "$script_dir/../ci/cargo-release-group.sh" version "${RELEASE_GROUP:-}")" || exit 1
commit="$(git rev-parse --verify HEAD)"
exec bash "$script_dir/../ci/check-release-tag.sh" "$commit" "$version"
