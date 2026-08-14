#!/usr/bin/env bash
set -euo pipefail

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-dependency-check.XXXXXX")"
trap 'rm -rf -- "${work_dir}"' EXIT

# Always start from one coherent RustSec snapshot. cargo-audit's shared cache
# can retain files removed by upstream advisory moves, causing unrelated parse
# failures before the workspace lockfile is inspected.
cargo audit --db "${work_dir}/advisory-db" --deny warnings \
  --ignore RUSTSEC-2021-0127 \
  --ignore RUSTSEC-2024-0436

cargo machete --with-metadata
