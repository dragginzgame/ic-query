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
  ic-wire
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

forbidden_ic_state_host_dependencies=(
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

forbidden_cmc_host_dependencies=(
  cap-fs-ext
  cap-std
  ic-host-artifacts
  ic-host-fs
  ic-host-http
  ic-host-process
  prost
)

forbidden_cloud_engine_host_dependencies=(
  cap-fs-ext
  cap-std
  ic-host-artifacts
  ic-host-fs
  ic-host-http
  ic-host-process
  prost
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

# Case selection is shared with compilation; assertions remain graph-specific.
while IFS=$'	' read -r target feature tests; do
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
        -- \
        --package ic-query \
        --target wasm32-unknown-unknown \
        --no-default-features
      ;;
    wasm32-unknown-unknown:canister)
      check_tree_absent "ic-query wasm32-unknown-unknown --features canister" \
        "${forbidden_canister_dependencies[@]}" \
        "${forbidden_tooling_dependencies[@]}" \
        -- \
        --package ic-query \
        --target wasm32-unknown-unknown \
        --no-default-features \
        --features canister
      ;;
    native:host)
      check_tree_absent "ic-query --features host --no-default-features" \
        "${forbidden_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features host
      ;;
    native:dashboard-host)
      check_tree_absent "ic-query --features dashboard-host" \
        "${forbidden_dashboard_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features dashboard-host
      check_tree_absent "ic-query --features dashboard-host direct dependencies" \
        "${forbidden_direct_dashboard_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features dashboard-host \
        --depth 1
      ;;
    native:ic-state-host)
      check_tree_absent "ic-query --features ic-state-host" \
        "${forbidden_ic_state_host_dependencies[@]}" \
        "${forbidden_agent_only_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features ic-state-host
      check_tree_absent "ic-query --features ic-state-host direct dependencies" \
        "${forbidden_direct_ic_state_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features ic-state-host \
        --depth 1
      ;;
    native:icrc-host)
      check_tree_absent "ic-query --features icrc-host" \
        "${forbidden_icrc_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features icrc-host
      check_tree_absent "ic-query --features icrc-host direct dependencies" \
        "${forbidden_direct_icrc_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features icrc-host \
        --depth 1
      ;;
    native:certificate-host)
      check_tree_absent "ic-query --features certificate-host" \
        "${forbidden_cmc_host_dependencies[@]}" \
        "${forbidden_host_dependencies[@]}" \
        ic-host-tools/ledger-evidence \
        -- \
        --package ic-query \
        --no-default-features \
        --features certificate-host
      ;;
    native:ledger-evidence-host)
      check_tree_absent "ic-query --features ledger-evidence-host" \
        "${forbidden_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features ledger-evidence-host
      ;;
    native:cmc-host)
      check_tree_absent "ic-query --features cmc-host" \
        "${forbidden_cmc_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features cmc-host
      check_tree_absent "ic-query --features cmc-host direct dependencies" \
        "${forbidden_direct_cmc_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features cmc-host \
        --depth 1
      ;;
    native:cloud-engine-host)
      check_tree_absent "ic-query --features cloud-engine-host" \
        "${forbidden_cloud_engine_host_dependencies[@]}" \
        "${forbidden_agent_only_host_dependencies[@]}" \
        ic-host-tools/encoding \
        -- \
        --package ic-query \
        --no-default-features \
        --features cloud-engine-host
      check_tree_absent "ic-query --features cloud-engine-host direct dependencies" \
        "${forbidden_direct_cloud_engine_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features cloud-engine-host \
        --depth 1
      ;;
    native:nns-host)
      check_tree_absent "ic-query --features nns-host direct dependencies" \
        "${forbidden_direct_nns_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features nns-host \
        --depth 1
      ;;
    native:certified-subnet-catalog-host)
      check_tree_absent "ic-query --features certified-subnet-catalog-host direct dependencies" \
        "${forbidden_direct_nns_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features certified-subnet-catalog-host \
        --depth 1
      ;;
    native:sns-host)
      check_tree_absent "ic-query --features sns-host" \
        "${forbidden_sns_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features sns-host
      check_tree_absent "ic-query --features sns-host direct dependencies" \
        "${forbidden_direct_sns_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features sns-host \
        --depth 1
      ;;
    native:subnet-catalog-host)
      check_tree_absent "ic-query --features subnet-catalog-host direct dependencies" \
        "${forbidden_direct_registry_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features subnet-catalog-host \
        --depth 1
      ;;
    native:nns-topology-host)
      check_tree_absent "ic-query --features nns-topology-host direct dependencies" \
        "${forbidden_direct_registry_host_dependencies[@]}" \
        -- \
        --package ic-query \
        --no-default-features \
        --features nns-topology-host \
        --depth 1
      ;;
    *) echo "error: no dependency policy for $target:$feature" >&2; exit 1 ;;
  esac
done < "$repo_root/ci/library-feature-cases.tsv"

check_tree_absent "ic-query-cli production dependencies" \
  "${forbidden_tooling_dependencies[@]}" \
  -- --package ic-query-cli
