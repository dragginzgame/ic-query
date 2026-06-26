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

run_quiet() {
  local label="$1"
  shift

  local log
  log="$(mktemp "${TMPDIR:-/tmp}/ic-query-feature-boundary.XXXXXX")"
  if ! "$@" >"${log}" 2>&1; then
    echo "error: ${label} failed" >&2
    cat "${log}" >&2
    rm -f "${log}"
    return 1
  fi
  rm -f "${log}"
}

cargo check -p ic-query --locked
cargo check -p ic-query --no-default-features --locked
cargo check -p ic-query --target wasm32-unknown-unknown --no-default-features --locked
run_quiet "ic-query --features host" \
  cargo check -p ic-query --no-default-features --features host --locked
cargo test -p ic-query --test icrc_public_api --no-default-features --locked
cargo test -p ic-query --test nns_public_api --no-default-features --locked
cargo test -p ic-query --test sns_public_api --no-default-features --locked
cargo test -p ic-query --test subnet_catalog_public_api --no-default-features --locked
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
