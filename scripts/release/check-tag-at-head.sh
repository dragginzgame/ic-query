#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
version="$(perl "$script_dir/metadata.pl" version)"
commit="$(git rev-parse --verify HEAD)"
exec bash "$script_dir/../ci/check-release-tag.sh" "$commit" "$version"
