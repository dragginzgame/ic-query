#!/usr/bin/env bash
set -euo pipefail
repo_root="$(CDPATH="" cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
native_target="$(rustc -vV | sed -n 's/^host: //p')"
[[ -n "$native_target" ]] || { echo 'missing compiler host target' >&2; exit 1; }

forbidden_tooling_dependencies=(
  futures-executor
  ic-build
  ic-host-process
  ic-testkit
  wat
)

forbidden_pure_library_dependencies=(
  candid_parser
  clap
  futures-util
  ic-agent
  ic-certification
  icrc-ledger-types
  ic-host-artifacts
  ic-host-fs
  ic-host-http
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
  futures-util
  ic-agent
  ic-certification
  icrc-ledger-types
  ic-host-artifacts
  ic-host-fs
  ic-host-http
  ic-host-process
  ic-host-tools
  prost
  reqwest
  tokio
)

# Certificate consumers select Host without Candid service parsing; retain
# interface, process and archive exclusions alongside parser exclusion.
forbidden_host_dependencies=(
  futures-executor
  candid_parser
  ic-host-tools/candid
  clap
  ic-host-process
  ic-host-artifacts/archive
  ic-host-artifacts/gzip
  ic-host-artifacts/wasm
  ic-host-tools/ic-interface
)

forbidden_direct_registry_host_dependencies=(
  ic-host-http
  reqwest
  serde_cbor
)

forbidden_dashboard_host_dependencies=(
  ic-agent
  prost
  serde_cbor
)

forbidden_direct_dashboard_host_dependencies=(
  futures-util
  ic-agent
  prost
  serde_cbor
  sha2
  tokio
)

forbidden_stateless_host_dependencies=(
  cap-fs-ext
  cap-std
  ic-host-artifacts
  ic-host-fs
  ic-host-http
  ic-host-process
  prost
)

forbidden_direct_ic_state_host_dependencies=(
  cap-fs-ext
  cap-std
  futures-util
  prost
  reqwest
  serde_cbor
  sha2
  tokio
)

forbidden_icrc_host_dependencies=(
  ic-host-http
  prost
)

forbidden_direct_icrc_host_dependencies=(
  prost
  reqwest
  tokio
)

forbidden_agent_only_host_dependencies=(
  candid_parser
  ic-host-tools/candid
  ic-host-tools/certificate
  ic-host-tools/ic-interface
  ic-host-tools/icp-project
  ic-host-tools/ledger-evidence
  ic-host-tools/registry
)

forbidden_direct_cmc_host_dependencies=(
  cap-fs-ext
  cap-std
  futures-util
  prost
  reqwest
  sha2
  tokio
)

forbidden_direct_cloud_engine_host_dependencies=(
  cap-fs-ext
  cap-std
  futures-util
  prost
  reqwest
  serde_cbor
  sha2
  tokio
)

forbidden_direct_nns_host_dependencies=(
  ic-host-http
  reqwest
)

forbidden_sns_host_dependencies=(
  ic-host-http
  prost
)

forbidden_direct_sns_host_dependencies=(
  prost
  reqwest
  serde_cbor
  sha2
  tokio
)

check_tree_absent() {
  local label="$1"
  shift
  # Exclude the test-only Futures umbrella, stable memory and scoped tempfiles.
  local -a rules=(--forbid futures --forbid ic-stable-structures --forbid ic-cdk --forbid ic-cdk-macros --forbid-feature ic-host-fs/temporary-file)
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
  if perl "$repo_root/scripts/ci/check-cargo-graph.pl" --consumer "$repo_root" \
    --target "$native_target" "${rules[@]}" "$@"; then
    return 0
  else
    local status="$?"
    printf 'error: dependency graph check failed: %s\n' "$label" >&2
    return "$status"
  fi
}

check_feature_graph() {
  local label="$1" feature="$2" depth="$3"
  shift 3
  local -a selection=(--package ic-query --no-default-features --features "$feature")
  [[ "$depth" == all ]] || selection+=(--depth "$depth")
  check_tree_absent "$label" "$@" -- "${selection[@]}"
}

# Case selection is shared with compilation; assertions remain graph-specific.
while IFS=$'	' read -r target feature _tests; do
  [[ -n "$target" && "$target" != \#* ]] || continue
  case "$target:$feature" in
    native:none)
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
      ;;
    native:default)
      check_tree_absent "ic-query default features" \
        "${forbidden_pure_library_dependencies[@]}" \
        -- \
        --package ic-query
      ;;
    wasm32-unknown-unknown:none)
      check_tree_absent "ic-query wasm32-unknown-unknown --no-default-features" \
        "${forbidden_pure_library_dependencies[@]}" \
        ic-wire \
        -- \
        --package ic-query \
        --target wasm32-unknown-unknown \
        --no-default-features
      ;;
    wasm32-unknown-unknown:canister)
      check_tree_absent "ic-query wasm32-unknown-unknown --features canister" \
        "${forbidden_canister_dependencies[@]}" \
        "${forbidden_tooling_dependencies[@]}" \
        ic-wire \
        -- \
        --package ic-query \
        --target wasm32-unknown-unknown \
        --no-default-features \
        --features canister
      ;;
    native:host)
      check_feature_graph "ic-query --features host --no-default-features" host all \
        "${forbidden_host_dependencies[@]}"
      ;;
    native:dashboard-host)
      check_feature_graph "ic-query --features dashboard-host" dashboard-host all \
        "${forbidden_dashboard_host_dependencies[@]}"
      check_feature_graph "ic-query --features dashboard-host direct dependencies" dashboard-host 1 \
        "${forbidden_direct_dashboard_host_dependencies[@]}"
      ;;
    native:ic-state-host)
      check_feature_graph "ic-query --features ic-state-host" ic-state-host all \
        "${forbidden_stateless_host_dependencies[@]}" \
        "${forbidden_agent_only_host_dependencies[@]}"
      check_feature_graph "ic-query --features ic-state-host direct dependencies" ic-state-host 1 \
        "${forbidden_direct_ic_state_host_dependencies[@]}"
      ;;
    native:icrc-host)
      check_feature_graph "ic-query --features icrc-host" icrc-host all \
        "${forbidden_icrc_host_dependencies[@]}"
      check_feature_graph "ic-query --features icrc-host direct dependencies" icrc-host 1 \
        "${forbidden_direct_icrc_host_dependencies[@]}"
      ;;
    native:certificate-host)
      check_feature_graph "ic-query --features certificate-host" certificate-host all \
        "${forbidden_stateless_host_dependencies[@]}" \
        "${forbidden_host_dependencies[@]}" \
        ic-host-tools/ledger-evidence
      ;;
    native:ledger-evidence-host)
      check_feature_graph "ic-query --features ledger-evidence-host" ledger-evidence-host all \
        "${forbidden_host_dependencies[@]}"
      ;;
    native:cmc-host)
      check_feature_graph "ic-query --features cmc-host" cmc-host all \
        "${forbidden_stateless_host_dependencies[@]}"
      check_feature_graph "ic-query --features cmc-host direct dependencies" cmc-host 1 \
        "${forbidden_direct_cmc_host_dependencies[@]}"
      ;;
    native:cloud-engine-host)
      check_feature_graph "ic-query --features cloud-engine-host" cloud-engine-host all \
        "${forbidden_stateless_host_dependencies[@]}" \
        "${forbidden_agent_only_host_dependencies[@]}" \
        ic-host-tools/encoding
      check_feature_graph "ic-query --features cloud-engine-host direct dependencies" cloud-engine-host 1 \
        "${forbidden_direct_cloud_engine_host_dependencies[@]}"
      ;;
    native:nns-host)
      check_feature_graph "ic-query --features nns-host direct dependencies" nns-host 1 \
        "${forbidden_direct_nns_host_dependencies[@]}"
      ;;
    native:certified-subnet-catalog-host)
      check_feature_graph "ic-query --features certified-subnet-catalog-host direct dependencies" certified-subnet-catalog-host 1 \
        "${forbidden_direct_nns_host_dependencies[@]}"
      ;;
    native:sns-host)
      check_feature_graph "ic-query --features sns-host" sns-host all \
        "${forbidden_sns_host_dependencies[@]}"
      check_feature_graph "ic-query --features sns-host direct dependencies" sns-host 1 \
        "${forbidden_direct_sns_host_dependencies[@]}"
      ;;
    native:subnet-catalog-host)
      check_feature_graph "ic-query --features subnet-catalog-host direct dependencies" subnet-catalog-host 1 \
        "${forbidden_direct_registry_host_dependencies[@]}"
      ;;
    native:nns-topology-host)
      check_feature_graph "ic-query --features nns-topology-host direct dependencies" nns-topology-host 1 \
        "${forbidden_direct_registry_host_dependencies[@]}"
      ;;
    *) echo "error: no dependency policy for $target:$feature" >&2; exit 1 ;;
  esac
done < "$repo_root/ci/library-feature-cases.tsv"

# Metrics supplies native Wire metadata; Wasm production graphs exclude Wire above.
check_tree_absent "ic-query-cli production dependencies" \
  "${forbidden_tooling_dependencies[@]}" \
  -- --package ic-query-cli
