#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${repo_root}"

group=workspace
if [[ "${1:-}" == --group ]]; then
  group="${2:-}"
  case "$group" in query|cli) ;; *) echo 'select package group query or cli' >&2; exit 2 ;; esac
  shift 2
fi

if [[ "$group" != cli ]]; then
  cargo package -p ic-query --locked "$@"
fi
if [[ "$group" != query ]]; then
  cargo package -p ic-query-text --locked \
    --config 'patch.crates-io.ic-query.path="crates/ic-query"' "$@"
  cargo package -p ic-query-cli --locked \
    --config 'patch.crates-io.ic-query.path="crates/ic-query"' --config 'patch.crates-io.ic-query-text.path="crates/ic-query-text"' "$@"
fi
