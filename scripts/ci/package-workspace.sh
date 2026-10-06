#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${repo_root}"

cargo package -p ic-query --locked "$@"
cargo package -p ic-query-cli --locked \
  --config 'patch.crates-io.ic-query.path="crates/ic-query"' "$@"
