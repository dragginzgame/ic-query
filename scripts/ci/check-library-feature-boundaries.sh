#!/usr/bin/env bash
set -euo pipefail

forbidden_library_boundary_dependencies=(
  clap
  futures
  ic-agent
  tokio
)

check_tree_absent() {
  local label="$1"
  shift

  local tree
  tree="$(cargo tree "$@" -e features)"

  local failed=0
  for dependency in "${forbidden_library_boundary_dependencies[@]}"; do
    if grep -Fq "${dependency}" <<<"${tree}"; then
      echo "error: ${label} unexpectedly includes ${dependency}" >&2
      failed=1
    fi
  done

  if [[ "${failed}" -ne 0 ]]; then
    echo "${tree}" >&2
    return 1
  fi
}

cargo check -p ic-query --locked
cargo check -p ic-query --no-default-features --locked
cargo check -p ic-query --target wasm32-unknown-unknown --no-default-features --locked
cargo check -p ic-query-cli --locked

check_tree_absent "ic-query --no-default-features" \
  -p ic-query \
  --no-default-features

check_tree_absent "ic-query default features" \
  -p ic-query

check_tree_absent "ic-query wasm32-unknown-unknown --no-default-features" \
  -p ic-query \
  --target wasm32-unknown-unknown \
  --no-default-features
