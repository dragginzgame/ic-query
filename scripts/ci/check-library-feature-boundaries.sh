#!/usr/bin/env bash
set -euo pipefail

forbidden_pure_library_dependencies=(
  clap
  futures
  ic-agent
  reqwest
  tokio
)

forbidden_direct_pure_library_dependencies=(
  prost
  sha2
  time
)

forbidden_host_dependencies=(
  clap
)

forbidden_direct_subnet_catalog_host_dependencies=(
  reqwest
  serde_cbor
)

check_tree_absent() {
  local label="$1"
  shift
  local -a dependencies=()
  while [[ "$#" -gt 0 && "$1" != "--" ]]; do
    dependencies+=("$1")
    shift
  done
  if [[ "$#" -eq 0 ]]; then
    echo "error: ${label} check is missing -- before cargo tree arguments" >&2
    return 1
  fi
  shift

  local tree
  tree="$(cargo tree "$@" -e features)"

  local failed=0
  for dependency in "${dependencies[@]}"; do
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
run_quiet "ic-query --features subnet-catalog-host" \
  cargo check -p ic-query --no-default-features --features subnet-catalog-host --locked
cargo test -p ic-query --test downstream_usage --no-default-features --locked
cargo test -p ic-query --test downstream_usage --no-default-features --features host --locked
cargo test -p ic-query --test icrc_public_api --no-default-features --locked
cargo test -p ic-query --test icrc_public_api --no-default-features --features host --locked
cargo test -p ic-query --test ic_public_api --no-default-features --locked
cargo test -p ic-query --test ic_public_api --no-default-features --features host --locked
cargo test -p ic-query --test nns_public_api --no-default-features --locked
cargo test -p ic-query --test nns_public_api --no-default-features --features host --locked
cargo test -p ic-query --test sns_public_api --no-default-features --locked
cargo test -p ic-query --test sns_public_api --no-default-features --features host --locked
cargo test -p ic-query --test subnet_catalog_public_api --no-default-features --locked
cargo test -p ic-query --test subnet_catalog_public_api --no-default-features --features subnet-catalog-host --locked
cargo test -p ic-query --test subnet_catalog_public_api --no-default-features --features host --locked
cargo test -p ic-query --test system_public_api --no-default-features --locked
cargo test -p ic-query --test system_public_api --no-default-features --features host --locked
cargo check -p ic-query-cli --locked

check_tree_absent "ic-query --no-default-features" \
  "${forbidden_pure_library_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features

check_tree_absent "ic-query --no-default-features direct dependencies" \
  "${forbidden_direct_pure_library_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  -e normal \
  --depth 1

check_tree_absent "ic-query default features" \
  "${forbidden_pure_library_dependencies[@]}" \
  -- \
  -p ic-query

check_tree_absent "ic-query wasm32-unknown-unknown --no-default-features" \
  "${forbidden_pure_library_dependencies[@]}" \
  -- \
  -p ic-query \
  --target wasm32-unknown-unknown \
  --no-default-features

check_tree_absent "ic-query --features host --no-default-features" \
  "${forbidden_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features host

# `ic-agent` may retain these package names transitively. The focused feature
# promises only that ic-query's own optional transport/certification edges stay
# disabled, so this gate intentionally inspects direct normal dependencies.
check_tree_absent "ic-query --features subnet-catalog-host direct dependencies" \
  "${forbidden_direct_subnet_catalog_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features subnet-catalog-host \
  -e normal \
  --depth 1
