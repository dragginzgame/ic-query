#!/usr/bin/env bash
set -euo pipefail

repo_root="$(CDPATH="" cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-feature-boundary.XXXXXX")"
work_dir="$(CDPATH="" cd -P -- "$work_dir" && printf '%s/.' "$PWD")"
work_dir="${work_dir%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "Feature boundary diagnostics retained"' EXIT
export TMPDIR="${work_dir%/*}"

run_quiet() {
  local label="$1"
  shift

  local log status
  log="$(mktemp "${work_dir}/check.XXXXXX")"
  if "$@" >"${log}" 2>&1; then
    return 0
  else
    status="$?"
    echo "error: ${label} failed" >&2
    cat "${log}" >&2
    return "${status}"
  fi
}

check_public_api() {
  local feature="$1" test
  shift
  local -a arguments=(test -p ic-query --no-default-features --locked --offline)
  if [[ "$feature" != none ]]; then
    arguments+=(--features "$feature")
  fi
  for test in "$@"; do
    arguments+=(--test "$test")
  done
  cargo "${arguments[@]}"
}

cargo check -p ic-query --locked --offline
cargo check -p ic-query --no-default-features --locked --offline
cargo check -p ic-query --target wasm32-unknown-unknown --no-default-features --locked --offline
run_quiet "ic-query wasm32-unknown-unknown --features canister" \
  cargo check -p ic-query --target wasm32-unknown-unknown --no-default-features --features canister --locked --offline
run_quiet "ic-query deployable Governance probe" \
  cargo check -p ic-query --example governance_probe --target wasm32-unknown-unknown --no-default-features --features canister --locked --offline
run_quiet "ic-query nns canister public API" \
  cargo check -p ic-query --test nns_public_api --target wasm32-unknown-unknown --no-default-features --features canister --locked --offline
run_quiet "ic-query --features host" \
  cargo check -p ic-query --no-default-features --features host --locked --offline
run_quiet "ic-query --features certificate-host" \
  cargo check -p ic-query --no-default-features --features certificate-host --locked --offline
run_quiet "ic-query --features ledger-evidence-host" \
  cargo check -p ic-query --no-default-features --features ledger-evidence-host --locked --offline
run_quiet "ic-query --features dashboard-host" \
  cargo check -p ic-query --no-default-features --features dashboard-host --locked --offline
run_quiet "ic-query --features ic-state-host" \
  cargo check -p ic-query --no-default-features --features ic-state-host --locked --offline
run_quiet "ic-query --features icrc-host" \
  cargo check -p ic-query --no-default-features --features icrc-host --locked --offline
run_quiet "ic-query --features subnet-catalog-host" \
  cargo check -p ic-query --no-default-features --features subnet-catalog-host --locked --offline
run_quiet "ic-query --features certified-subnet-catalog-host" \
  cargo check -p ic-query --no-default-features --features certified-subnet-catalog-host --locked --offline
run_quiet "ic-query --features nns-topology-host" \
  cargo check -p ic-query --no-default-features --features nns-topology-host --locked --offline
run_quiet "ic-query --features nns-host" \
  cargo check -p ic-query --no-default-features --features nns-host --locked --offline
run_quiet "ic-query --features sns-host" \
  cargo check -p ic-query --no-default-features --features sns-host --locked --offline
run_quiet "ic-query --features cmc-host" \
  cargo check -p ic-query --no-default-features --features cmc-host --locked --offline
run_quiet "ic-query --features cloud-engine-host" \
  cargo check -p ic-query --no-default-features --features cloud-engine-host --locked --offline
common_public_api_tests=(
  downstream_usage
  icrc_public_api
  ic_public_api
  cloud_engine_public_api
  nns_public_api
  sns_public_api
  subnet_catalog_public_api
  subnet_topology_public_api
  system_public_api
)
check_public_api none "${common_public_api_tests[@]}"
check_public_api host "${common_public_api_tests[@]}"
check_public_api certificate-host downstream_usage
check_public_api icrc-host icrc_public_api
check_public_api dashboard-host ic_public_api cloud_engine_public_api
check_public_api ic-state-host ic_public_api
check_public_api cloud-engine-host cloud_engine_public_api
check_public_api certified-subnet-catalog-host certified_subnet_catalog_public_api
check_public_api nns-host nns_public_api
check_public_api sns-host sns_public_api
check_public_api subnet-catalog-host subnet_catalog_public_api subnet_topology_public_api
check_public_api nns-topology-host subnet_topology_public_api
check_public_api cmc-host system_public_api
cargo check -p ic-query-cli --locked --offline

bash "$repo_root/scripts/ci/check-library-dependency-graphs.sh"
fixture_complete=true
