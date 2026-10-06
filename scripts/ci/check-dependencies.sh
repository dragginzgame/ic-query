#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-dependency-check.XXXXXX")"
finish() {
  local status=$?
  if [[ "$status" == 0 ]]; then
    rm -rf -- "$work_dir"
  else
    echo "Dependency check failed; evidence retained: $work_dir" >&2
  fi
}
trap finish EXIT

# Always start from one coherent RustSec snapshot. cargo-audit's shared cache
# can retain files removed by upstream advisory moves, causing unrelated parse
# failures before the workspace lockfile is inspected.
echo "Fetching a fresh RustSec snapshot (shallow clone; Git progress follows)" >&2
bash "$repo_root/scripts/ci/prepare-rustsec-db.sh" online \
  https://github.com/RustSec/advisory-db.git "$work_dir/rustsec" > /dev/null

cargo audit --no-fetch --db "${work_dir}/rustsec/db" --deny warnings \
  --ignore RUSTSEC-2021-0127 \
  --ignore RUSTSEC-2024-0436

CARGO_NET_OFFLINE=true cargo machete --with-metadata
