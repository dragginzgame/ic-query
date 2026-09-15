#!/usr/bin/env bash
set -euo pipefail

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-dependency-check.XXXXXX")"
trap 'rm -rf -- "${work_dir}"' EXIT

# Always start from one coherent RustSec snapshot. cargo-audit's shared cache
# can retain files removed by upstream advisory moves, causing unrelated parse
# failures before the workspace lockfile is inspected.
echo "Fetching a fresh RustSec snapshot (shallow clone; Git progress follows)" >&2
GIT_TERMINAL_PROMPT=0 git -c http.lowSpeedLimit=1024 -c http.lowSpeedTime=30 \
  clone --depth 1 --single-branch --no-tags --progress \
  https://github.com/RustSec/advisory-db.git "${work_dir}/advisory-db"

cargo audit --no-fetch --db "${work_dir}/advisory-db" --deny warnings \
  --ignore RUSTSEC-2021-0127 \
  --ignore RUSTSEC-2024-0436

cargo machete --with-metadata
