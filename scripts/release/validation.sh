#!/usr/bin/env bash
# Shared companions: scripts/release/select-validation.pl scripts/ci/validation-evidence.pl scripts/ci/release-receipts.sh scripts/ci/run-validation-targets.sh scripts/ci/check-release-source.sh scripts/ci/cargo-release-group.sh
set -euo pipefail
root="$(CDPATH='' cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$root"
for directory in target target/release-validation; do
  [[ ! -L "$directory" && ( ! -e "$directory" || -d "$directory" ) ]] \
    || { echo 'validation output directory is redirected' >&2; exit 1; }
done
# shellcheck source=scripts/ci/release-receipts.sh
source scripts/ci/release-receipts.sh
operation="${1:-}"
attempt="${2:?validation attempt required}"
observe_baseline() {
  perl scripts/ci/validation-evidence.pl baseline "$root" "$RELEASE_SOURCE" "refs/tags/$RELEASE_GROUP-v$RELEASE_PREVIOUS"
}
context() {
  perl scripts/release/select-validation.pl context "$root" "$1"
}
custody() {
  printf 'query-group-validation-1\n'
  release_receipt_identity
  local paths=() path
  perl scripts/release/select-validation.pl custody-paths "$root" "$gate" > "$gate/custody-paths"
  while IFS= read -r -d '' path; do paths+=("$path"); done < "$gate/custody-paths"
  release_receipt_payload '' "${paths[@]}"
}
case "$operation" in
  run|qualify)
    mkdir -p target/release-validation
    gate="$(umask 077
      mktemp -d target/release-validation/run.XXXXXX)"
    purpose=release mode=release
    if [[ "$operation" == qualify ]]; then
      purpose=qualification
      mode="${3:-qualification}"
    fi
    bash scripts/ci/check-release-source.sh
    [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || { echo 'validation source changed' >&2; exit 1; }
    # Canonical Cargo admission owns versions, member identities and group selection.
    bash scripts/ci/cargo-release-group.sh version "$RELEASE_GROUP" > "$gate/version"
    [[ "$(cat "$gate/version")" == "$RELEASE_PREVIOUS" ]] || { echo 'validation group version differs' >&2; exit 1; }
    jq -cS '[.groups[].name]' ci/cargo-release-groups.json > "$gate/groups.json"
    perl scripts/ci/validation-evidence.pl roster "$root" ci/release-validation.json "$gate/groups.json" > "$gate/full-stages"
    observe_baseline > "$gate/baseline.json"
    context "$purpose" > "$gate/context.json"
    perl scripts/release/select-validation.pl plan "$root" "$gate" "$mode" > "$gate/plan.json"
    mkdir "$gate/logs"
    VALIDATION_LOG_DIR="$root/$gate/logs" CARGO_NET_OFFLINE=true \
      bash scripts/ci/run-validation-targets.sh --plan "$gate/plan.json"
    selected_run=''
    for candidate in "$gate/logs/"validation.*; do
      parsed="$candidate/parsed-inputs.json.plan.json"
      if [[ -f "$parsed" && ! -L "$parsed" ]] && cmp -s "$gate/plan.json" "$parsed"; then
        [[ -z "$selected_run" ]] || { echo 'ambiguous validation run' >&2; exit 1; }
        selected_run="$candidate"
      fi
    done
    [[ -n "$selected_run" ]] || { echo 'matching validation run is missing' >&2; exit 1; }
    release_receipt_write "$gate/proof.json" perl scripts/ci/validation-evidence.pl verify \
      "$root" "$gate/plan.json" "$RELEASE_SOURCE" "$RELEASE_GROUP" "$selected_run" "$gate/context.json"
    bash scripts/ci/check-release-source.sh
    if [[ "$operation" == run ]]; then
      release_receipt_write "$gate/custody" custody
      printf '%s\n' "$gate" > "$attempt/group-validation"
    else
      printf '%s\n' "$gate" > "$attempt"
    fi
    ;;
  check)
    IFS= read -r gate < "$attempt/group-validation"
    [[ "$gate" =~ ^target/release-validation/run\.[A-Za-z0-9]+$ && -d "$gate" && ! -L "$gate" ]] \
      || { echo 'invalid group validation pointer' >&2; exit 1; }
    context release > "$gate/context-observation.json"
    cmp "$gate/context.json" "$gate/context-observation.json"
    observe_baseline > "$gate/baseline-observation.json"
    cmp "$gate/baseline.json" "$gate/baseline-observation.json"
    release_receipt_check "$gate/custody" custody
    # Metadata admission authenticates the intentional version preparation.
    # The original verified runner evidence remains under exact byte custody.
    ;;
  *) echo 'usage: validation.sh run|check ATTEMPT | qualify POINTER workspace|qualification' >&2; exit 2 ;;
esac
