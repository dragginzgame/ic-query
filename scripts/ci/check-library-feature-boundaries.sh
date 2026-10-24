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

bash "$repo_root/scripts/ci/check-library-dependency-graphs.sh"

while IFS=$'	' read -r target feature tests; do
  [[ -n "$target" && "$target" != \#* ]] || continue
  arguments=(check -p ic-query --locked --offline)
  [[ "$target" == native ]] || arguments+=(--target "$target")
  [[ "$feature" == default ]] || arguments+=(--no-default-features)
  case "$feature" in default|none) ;; *) arguments+=(--features "$feature") ;; esac
  run_quiet "ic-query $target $feature" cargo "${arguments[@]}"
  if [[ "$tests" != - ]]; then
    read -r -a api_tests <<<"$tests"
    check_public_api "$feature" "${api_tests[@]}"
  fi
done < "$repo_root/ci/library-feature-cases.tsv"

run_quiet "ic-query deployable Governance probe"   cargo check -p ic-query --example governance_probe --target wasm32-unknown-unknown --no-default-features --features canister --locked --offline
run_quiet "ic-query nns canister public API"   cargo check -p ic-query --test nns_public_api --target wasm32-unknown-unknown --no-default-features --features canister --locked --offline
cargo check -p ic-query-cli --locked --offline

fixture_complete=true
