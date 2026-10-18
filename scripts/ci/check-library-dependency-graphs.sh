#!/usr/bin/env bash
set -euo pipefail
repo_root="$(CDPATH="" cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
native_target="$(rustc -vV | sed -n 's/^host: //p')"
[[ -n "$native_target" ]] || { echo 'missing compiler host target' >&2; exit 1; }

forbidden_tooling_dependencies=(
  ic-build
  ic-host-process
  ic-testkit
  wat
)

forbidden_pure_library_dependencies=(
  candid_parser
  clap
  futures
  ic-agent
  ic-certification
  icrc-ledger-types
  ic-host-artifacts
  ic-host-fs
  ic-host-process
  ic-host-tools
  reqwest
  tokio
)

forbidden_direct_pure_library_dependencies=(
  prost
  sha2
  time
)

forbidden_canister_dependencies=(
  candid_parser
  cap-fs-ext
  cap-std
  clap
  futures
  ic-agent
  ic-certification
  icrc-ledger-types
  ic-host-artifacts
  ic-host-fs
  ic-host-process
  ic-host-tools
  prost
  reqwest
  tokio
)

# Host Tools includes its Candid parser unconditionally; certificate consumers
# still exclude interface extraction, process execution and archive decoding.
forbidden_host_dependencies=(
  clap
  ic-host-process
  ic-host-artifacts/archive
  ic-host-artifacts/gzip
  ic-host-artifacts/wasm
  ic-host-tools/ic-interface
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
  ic-host-artifacts
  ic-host-fs
  ic-host-process
  ic-host-tools
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
  ic-host-artifacts
  ic-host-fs
  ic-host-process
  prost
)

forbidden_cloud_engine_host_dependencies=(
  cap-fs-ext
  cap-std
  ic-host-artifacts
  ic-host-fs
  ic-host-process
  ic-host-tools
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
  # Query excludes stable-memory integration for every supported selection.
  local -a rules=(--forbid ic-stable-structures)
  while [[ "$#" -gt 0 && "$1" != "--" ]]; do
    if [[ "$1" == */* ]]; then
      rules+=(--forbid-feature "$1")
    else
      rules+=(--forbid "$1")
    fi
    shift
  done
  [[ "$#" -gt 0 ]] || { echo "missing graph arguments: $label" >&2; return 1; }
  shift
  perl "$repo_root/scripts/ci/check-cargo-graph.pl" --consumer "$repo_root" \
    --target "$native_target" "${rules[@]}" "$@"
}

check_tree_absent "ic-query --no-default-features" \
  "${forbidden_pure_library_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features

check_tree_absent "ic-query --no-default-features direct dependencies" \
  "${forbidden_direct_pure_library_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --depth 1

check_tree_absent "ic-query default features" \
  "${forbidden_pure_library_dependencies[@]}" \
  -- \
  --package ic-query

check_tree_absent "ic-query wasm32-unknown-unknown --no-default-features" \
  "${forbidden_pure_library_dependencies[@]}" \
  -- \
  --package ic-query \
  --target wasm32-unknown-unknown \
  --no-default-features

check_tree_absent "ic-query wasm32-unknown-unknown --features canister" \
  "${forbidden_canister_dependencies[@]}" \
  "${forbidden_tooling_dependencies[@]}" \
  -- \
  --package ic-query \
  --target wasm32-unknown-unknown \
  --no-default-features \
  --features canister

check_tree_absent "ic-query --features host --no-default-features" \
  "${forbidden_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features host

check_tree_absent "ic-query --features dashboard-host" \
  "${forbidden_dashboard_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features dashboard-host

# Reqwest may retain packages such as SHA-256 implementations transitively.
# This gate separately proves that ic-query does not activate its Registry,
# certification, or async-source dependencies directly for Dashboard use.
check_tree_absent "ic-query --features dashboard-host direct dependencies" \
  "${forbidden_direct_dashboard_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features dashboard-host \
  --depth 1

check_tree_absent "ic-query --features ic-state-host" \
  "${forbidden_ic_state_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features ic-state-host

# `ic-agent` retains Reqwest, CBOR, and cryptographic packages transitively.
# Certified IC state collection directly needs only the agent, timers,
# and endpoint validation; it must not activate cache, Dashboard, Registry, or
# independent certificate-decoding dependencies.
check_tree_absent "ic-query --features ic-state-host direct dependencies" \
  "${forbidden_direct_ic_state_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features ic-state-host \
  --depth 1

check_tree_absent "ic-query --features icrc-host" \
  "${forbidden_icrc_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features icrc-host

# `ic-agent` retains Reqwest transitively. The focused ICRC feature promises
# that ic-query does not activate its Dashboard transport or Registry protobuf
# dependency directly, not that those package names disappear transitively.
check_tree_absent "ic-query --features icrc-host direct dependencies" \
  "${forbidden_direct_icrc_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features icrc-host \
  --depth 1

check_tree_absent "ic-query --features certificate-host" \
  "${forbidden_cmc_host_dependencies[@]}" \
  "${forbidden_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features certificate-host

check_tree_absent "ic-query --features ledger-evidence-host" \
  "${forbidden_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features ledger-evidence-host

check_tree_absent "ic-query --features cmc-host" \
  "${forbidden_cmc_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features cmc-host

# `ic-agent` retains Reqwest and cryptographic packages transitively. The CMC
# feature directly enables CBOR because certificate and witness decoding is
# part of its authority contract, but it does not need cache, Registry, or
# direct hashing dependencies.
check_tree_absent "ic-query --features cmc-host direct dependencies" \
  "${forbidden_direct_cmc_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features cmc-host \
  --depth 1

check_tree_absent "ic-query --features cloud-engine-host" \
  "${forbidden_cloud_engine_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features cloud-engine-host

# `ic-agent` retains Reqwest, CBOR, and cryptographic packages transitively.
# CloudEngine queries need only the agent, timers, and URL validation;
# cache, Registry, and certification dependencies must remain disabled directly.
check_tree_absent "ic-query --features cloud-engine-host direct dependencies" \
  "${forbidden_direct_cloud_engine_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features cloud-engine-host \
  --depth 1

# The complete NNS feature intentionally includes Registry protobuf and
# hashing through `nns-topology-host`, plus direct CBOR decoding for certified
# Registry-version evidence. It must not activate Dashboard Reqwest transport.
check_tree_absent "ic-query --features nns-host direct dependencies" \
  "${forbidden_direct_nns_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features nns-host \
  --depth 1

# Certified catalog consumers need the Registry, cache, hashing, and CBOR
# edges but not ic-query's direct Dashboard Reqwest transport.
check_tree_absent "ic-query --features certified-subnet-catalog-host direct dependencies" \
  "${forbidden_direct_nns_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features certified-subnet-catalog-host \
  --depth 1

check_tree_absent "ic-query --features sns-host" \
  "${forbidden_sns_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features sns-host

# `ic-agent` retains Reqwest, CBOR, and cryptographic packages transitively.
# The focused SNS feature promises that ic-query does not activate its
# Dashboard transport, Registry protobuf, or native ICRC certification edges
# directly, not that those package names disappear transitively.
check_tree_absent "ic-query --features sns-host direct dependencies" \
  "${forbidden_direct_sns_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features sns-host \
  --depth 1

# `ic-agent` may retain these package names transitively. The focused feature
# promises only that ic-query's own optional transport/certification edges stay
# disabled, so this gate intentionally inspects direct normal dependencies.
check_tree_absent "ic-query --features subnet-catalog-host direct dependencies" \
  "${forbidden_direct_registry_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features subnet-catalog-host \
  --depth 1

check_tree_absent "ic-query --features nns-topology-host direct dependencies" \
  "${forbidden_direct_registry_host_dependencies[@]}" \
  -- \
  --package ic-query \
  --no-default-features \
  --features nns-topology-host \
  --depth 1

check_tree_absent "ic-query-cli production dependencies" \
  "${forbidden_tooling_dependencies[@]}" \
  -- --package ic-query-cli
