#!/usr/bin/env bash
set -euo pipefail

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-feature-boundary.XXXXXX")"
trap 'rm -rf -- "${work_dir}"' EXIT

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

forbidden_canister_dependencies=(
  cap-fs-ext
  cap-std
  clap
  futures
  ic-agent
  prost
  reqwest
  tokio
)

forbidden_host_dependencies=(
  clap
)

forbidden_direct_registry_host_dependencies=(
  reqwest
  serde_cbor
)

forbidden_dashboard_host_dependencies=(
  ic-agent
  prost
  serde_cbor
)

forbidden_direct_dashboard_host_dependencies=(
  futures
  ic-agent
  prost
  serde_cbor
  sha2
)

forbidden_ic_state_host_dependencies=(
  cap-fs-ext
  cap-std
  prost
)

forbidden_direct_ic_state_host_dependencies=(
  cap-fs-ext
  cap-std
  futures
  prost
  reqwest
  serde_cbor
  sha2
)

forbidden_icrc_host_dependencies=(
  prost
)

forbidden_direct_icrc_host_dependencies=(
  prost
  reqwest
)

forbidden_cmc_host_dependencies=(
  cap-fs-ext
  cap-std
  prost
)

forbidden_cloud_engine_host_dependencies=(
  cap-fs-ext
  cap-std
  prost
)

forbidden_direct_cmc_host_dependencies=(
  cap-fs-ext
  cap-std
  futures
  prost
  reqwest
  sha2
)

forbidden_direct_cloud_engine_host_dependencies=(
  cap-fs-ext
  cap-std
  futures
  prost
  reqwest
  serde_cbor
  sha2
)

forbidden_direct_nns_host_dependencies=(
  reqwest
)

forbidden_sns_host_dependencies=(
  prost
)

forbidden_direct_sns_host_dependencies=(
  prost
  reqwest
  serde_cbor
  sha2
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
  log="$(mktemp "${work_dir}/check.XXXXXX")"
  if ! "$@" >"${log}" 2>&1; then
    echo "error: ${label} failed" >&2
    cat "${log}" >&2
    return 1
  fi
}

cargo check -p ic-query --locked
cargo check -p ic-query --no-default-features --locked
cargo check -p ic-query --target wasm32-unknown-unknown --no-default-features --locked
run_quiet "ic-query wasm32-unknown-unknown --features canister" \
  cargo check -p ic-query --target wasm32-unknown-unknown --no-default-features --features canister --locked
run_quiet "ic-query nns canister public API" \
  cargo check -p ic-query --test nns_public_api --target wasm32-unknown-unknown --no-default-features --features canister --locked
run_quiet "ic-query --features host" \
  cargo check -p ic-query --no-default-features --features host --locked
run_quiet "ic-query --features dashboard-host" \
  cargo check -p ic-query --no-default-features --features dashboard-host --locked
run_quiet "ic-query --features ic-state-host" \
  cargo check -p ic-query --no-default-features --features ic-state-host --locked
run_quiet "ic-query --features icrc-host" \
  cargo check -p ic-query --no-default-features --features icrc-host --locked
run_quiet "ic-query --features subnet-catalog-host" \
  cargo check -p ic-query --no-default-features --features subnet-catalog-host --locked
run_quiet "ic-query --features certified-subnet-catalog-host" \
  cargo check -p ic-query --no-default-features --features certified-subnet-catalog-host --locked
run_quiet "ic-query --features nns-topology-host" \
  cargo check -p ic-query --no-default-features --features nns-topology-host --locked
run_quiet "ic-query --features nns-host" \
  cargo check -p ic-query --no-default-features --features nns-host --locked
run_quiet "ic-query --features sns-host" \
  cargo check -p ic-query --no-default-features --features sns-host --locked
run_quiet "ic-query --features cmc-host" \
  cargo check -p ic-query --no-default-features --features cmc-host --locked
run_quiet "ic-query --features cloud-engine-host" \
  cargo check -p ic-query --no-default-features --features cloud-engine-host --locked
cargo test -p ic-query --test downstream_usage --no-default-features --locked
cargo test -p ic-query --test downstream_usage --no-default-features --features host --locked
cargo test -p ic-query --test icrc_public_api --no-default-features --locked
cargo test -p ic-query --test icrc_public_api --no-default-features --features icrc-host --locked
cargo test -p ic-query --test icrc_public_api --no-default-features --features host --locked
cargo test -p ic-query --test ic_public_api --no-default-features --locked
cargo test -p ic-query --test ic_public_api --no-default-features --features dashboard-host --locked
cargo test -p ic-query --test ic_public_api --no-default-features --features ic-state-host --locked
cargo test -p ic-query --test ic_public_api --no-default-features --features host --locked
cargo test -p ic-query --test cloud_engine_public_api --no-default-features --locked
cargo test -p ic-query --test cloud_engine_public_api --no-default-features --features dashboard-host --locked
cargo test -p ic-query --test cloud_engine_public_api --no-default-features --features cloud-engine-host --locked
cargo test -p ic-query --test cloud_engine_public_api --no-default-features --features host --locked
cargo test -p ic-query --test nns_public_api --no-default-features --locked
cargo test -p ic-query --test certified_subnet_catalog_public_api --no-default-features --features certified-subnet-catalog-host --locked
cargo test -p ic-query --test nns_public_api --no-default-features --features nns-host --locked
cargo test -p ic-query --test nns_public_api --no-default-features --features host --locked
cargo test -p ic-query --test sns_public_api --no-default-features --locked
cargo test -p ic-query --test sns_public_api --no-default-features --features sns-host --locked
cargo test -p ic-query --test sns_public_api --no-default-features --features host --locked
cargo test -p ic-query --test subnet_catalog_public_api --no-default-features --locked
cargo test -p ic-query --test subnet_catalog_public_api --no-default-features --features subnet-catalog-host --locked
cargo test -p ic-query --test subnet_catalog_public_api --no-default-features --features host --locked
cargo test -p ic-query --test subnet_topology_public_api --no-default-features --locked
cargo test -p ic-query --test subnet_topology_public_api --no-default-features --features subnet-catalog-host --locked
cargo test -p ic-query --test subnet_topology_public_api --no-default-features --features nns-topology-host --locked
cargo test -p ic-query --test subnet_topology_public_api --no-default-features --features host --locked
cargo test -p ic-query --test system_public_api --no-default-features --locked
cargo test -p ic-query --test system_public_api --no-default-features --features cmc-host --locked
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

check_tree_absent "ic-query wasm32-unknown-unknown --features canister" \
  "${forbidden_canister_dependencies[@]}" \
  -- \
  -p ic-query \
  --target wasm32-unknown-unknown \
  --no-default-features \
  --features canister

check_tree_absent "ic-query --features host --no-default-features" \
  "${forbidden_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features host

check_tree_absent "ic-query --features dashboard-host" \
  "${forbidden_dashboard_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features dashboard-host

# Reqwest may retain packages such as SHA-256 implementations transitively.
# This gate separately proves that ic-query does not activate its Registry,
# certification, or async-source dependencies directly for Dashboard use.
check_tree_absent "ic-query --features dashboard-host direct dependencies" \
  "${forbidden_direct_dashboard_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features dashboard-host \
  -e normal \
  --depth 1

check_tree_absent "ic-query --features ic-state-host" \
  "${forbidden_ic_state_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features ic-state-host

# `ic-agent` retains Reqwest, CBOR, and cryptographic packages transitively.
# Certified IC state collection directly needs only the agent, runtime bridge,
# and endpoint validation; it must not activate cache, Dashboard, Registry, or
# independent certificate-decoding dependencies.
check_tree_absent "ic-query --features ic-state-host direct dependencies" \
  "${forbidden_direct_ic_state_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features ic-state-host \
  -e normal \
  --depth 1

check_tree_absent "ic-query --features icrc-host" \
  "${forbidden_icrc_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features icrc-host

# `ic-agent` retains Reqwest transitively. The focused ICRC feature promises
# that ic-query does not activate its Dashboard transport or Registry protobuf
# dependency directly, not that those package names disappear transitively.
check_tree_absent "ic-query --features icrc-host direct dependencies" \
  "${forbidden_direct_icrc_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features icrc-host \
  -e normal \
  --depth 1

check_tree_absent "ic-query --features cmc-host" \
  "${forbidden_cmc_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features cmc-host

# `ic-agent` retains Reqwest and cryptographic packages transitively. The CMC
# feature directly enables CBOR because certificate and witness decoding is
# part of its authority contract, but it does not need cache, Registry, or
# direct hashing dependencies.
check_tree_absent "ic-query --features cmc-host direct dependencies" \
  "${forbidden_direct_cmc_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features cmc-host \
  -e normal \
  --depth 1

check_tree_absent "ic-query --features cloud-engine-host" \
  "${forbidden_cloud_engine_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features cloud-engine-host

# `ic-agent` retains Reqwest, CBOR, and cryptographic packages transitively.
# CloudEngine queries need only the agent, runtime bridge, and URL validation;
# cache, Registry, and certification dependencies must remain disabled directly.
check_tree_absent "ic-query --features cloud-engine-host direct dependencies" \
  "${forbidden_direct_cloud_engine_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features cloud-engine-host \
  -e normal \
  --depth 1

# The complete NNS feature intentionally includes Registry protobuf and
# hashing through `nns-topology-host`, plus direct CBOR decoding for certified
# Registry-version evidence. It must not activate Dashboard Reqwest transport.
check_tree_absent "ic-query --features nns-host direct dependencies" \
  "${forbidden_direct_nns_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features nns-host \
  -e normal \
  --depth 1

# Certified catalog consumers need the Registry, cache, hashing, and CBOR
# edges but not ic-query's direct Dashboard Reqwest transport.
check_tree_absent "ic-query --features certified-subnet-catalog-host direct dependencies" \
  "${forbidden_direct_nns_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features certified-subnet-catalog-host \
  -e normal \
  --depth 1

check_tree_absent "ic-query --features sns-host" \
  "${forbidden_sns_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features sns-host

# `ic-agent` retains Reqwest, CBOR, and cryptographic packages transitively.
# The focused SNS feature promises that ic-query does not activate its
# Dashboard transport, Registry protobuf, or native ICRC certification edges
# directly, not that those package names disappear transitively.
check_tree_absent "ic-query --features sns-host direct dependencies" \
  "${forbidden_direct_sns_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features sns-host \
  -e normal \
  --depth 1

# `ic-agent` may retain these package names transitively. The focused feature
# promises only that ic-query's own optional transport/certification edges stay
# disabled, so this gate intentionally inspects direct normal dependencies.
check_tree_absent "ic-query --features subnet-catalog-host direct dependencies" \
  "${forbidden_direct_registry_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features subnet-catalog-host \
  -e normal \
  --depth 1

check_tree_absent "ic-query --features nns-topology-host direct dependencies" \
  "${forbidden_direct_registry_host_dependencies[@]}" \
  -- \
  -p ic-query \
  --no-default-features \
  --features nns-topology-host \
  -e normal \
  --depth 1
