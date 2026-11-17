#!/usr/bin/env bash
# Query release policy; shared runners own metadata transactions and delivery.
set -euo pipefail
[[ $# == 1 ]] || exit 2
root="$(git rev-parse --show-toplevel)"
[[ "$PWD" == "$root" ]] || { echo 'Run release hooks from the repository root.' >&2; exit 2; }
shared="$root/vendor/toko-tooling/scripts/ci"
export PATH="$root/.tools/host/bin:$root/.tools/rust/common/bin:$PATH"
mkdir -p "$root/target/tmp"
export TMPDIR="$root/target/tmp"
group="${RELEASE_GROUP:-}"
case "$group" in topology|sns-migration|query|cli) ;; *) echo 'Select RELEASE_GROUP=topology, sns-migration, query or cli.' >&2; exit 2 ;; esac
make_bin="${RELEASE_MAKE:-make}"
group_metadata() { bash "$shared/cargo-release-group.sh" "$1" "$group"; }
editor() {
    bash "$root/vendor/toko-tooling/scripts/dev/install-rust-tools.sh" --consumer "$root" \
        --package ic-build --version 0.73.0 --bin ic-build-release-metadata --profile debug --features cli "$@"
}
admit_source() {
    local plan path allowed=()
    plan="$(group_metadata plan)"
    while IFS= read -r path; do allowed+=(--allow "$path"); done < <(printf '%s' "$plan" | jq -r '.files[]')
    bash "$shared/check-release-source.sh" "${allowed[@]}"
}
state="$(git rev-parse --git-path release-state)"
[[ "$state" == /* ]] || state="$root/$state"
source "$shared/release-receipts.sh"
source "$shared/release-timing.sh"
trap 'release_timing_end "$?"' EXIT
validation="$state/$group-${RELEASE_VERSION:-unset}.query-validated"
payload="$state/$group-${RELEASE_VERSION:-unset}.query-payload"
selection="$state/$group-${RELEASE_VERSION:-unset}.query-selection"
# Old full-gate receipts remain valid only for their original pre-selection source.
# New receipts retain the shared proof and exact evidence bytes across metadata preparation.
validation_identity() {
    release_receipt_identity || return
    if git cat-file -e "${RELEASE_SOURCE:?}:scripts/ci/validation.sh" 2>/dev/null; then
        [[ -f "$selection" && ! -L "$selection" ]] || return 1
        local attempt proof listed paths=() path route part
        IFS= read -r attempt < "$selection"
        [[ "$attempt" =~ ^target/validation/release\.[A-Za-z0-9]+$ ]] || return 1
        proof="$attempt/proof.json"
        paths+=("$selection" "$proof" "$attempt/completed" "$attempt/plan.json" "$attempt/context.json" "$attempt/baseline.json" "$attempt/inventory.json")
        if [[ -f "$attempt/projection.json" ]]; then
            paths+=("$attempt/projection.json" "$attempt/reuse-environment.json")
            if [[ -f "$attempt/obligations.json" ]]; then
                paths+=("$attempt/obligations.json" "$attempt/execution-policy.json")
            fi
            paths+=("$attempt/products.seal.json")
            listed="$(jq -er '.files | keys[]' "$attempt/products.seal.json")" || return
            while IFS= read -r path; do paths+=("$path"); done <<< "$listed"
        fi
        listed="$(jq -er '.run as $run | .files | keys[] | $run + "/" + .' "$proof")" || return
        while IFS= read -r path; do paths+=("$path"); done <<< "$listed"
        printf 'query-selected-validation-v1\n' || return
        for path in "${paths[@]}"; do
            route=''
            local components=()
            IFS=/ read -r -a components <<< "$path"
            [[ "$path" != /* ]] || route=/
            for part in "${components[@]}"; do
                [[ -n "$part" ]] || continue
                [[ "$part" != . && "$part" != .. ]] || return 1
                route="${route:+$route/}$part"
                [[ ! -L "$route" ]] || return 1
            done
            [[ -f "$path" && ! -L "$path" ]] || return 1
            git hash-object -- "$path" || return
        done
    fi
}
payload_identity() {
    local plan files=() path
    release_receipt_identity
    plan="$(group_metadata plan)"
    while IFS= read -r path; do files+=("$path"); done < <(printf '%s' "$plan" | jq -r '.files[]')
    release_receipt_payload "${RELEASE_COMMIT:-}" "${files[@]}"
}
case "$1" in
    setup)
        bash "$shared/check-release-source.sh"
        release_timing_begin query-setup-dependencies
        if [[ "${RELEASE_SETUP_MODE:-check}" == prepare ]]; then
            cargo fetch --locked --offline || cargo fetch --locked
        else
            cargo fetch --locked --offline
        fi
        release_timing_end 0
        release_timing_begin query-setup-tools
        if ! "$make_bin" --no-print-directory tools-check; then
            [[ "${RELEASE_SETUP_MODE:-check}" == prepare ]] || exit 1
            "$make_bin" --no-print-directory install-tools
            "$make_bin" --no-print-directory tools-check
        fi
        if ! editor --check >/dev/null; then
            [[ "${RELEASE_SETUP_MODE:-check}" == prepare ]] || exit 1
            editor >/dev/null
            editor --check >/dev/null
        fi
        "$make_bin" --no-print-directory toko-tooling-check
        release_timing_end 0
        ;;
    version) group_metadata version ;;
    files) group_metadata files ;;
    validated) release_receipt_check "$validation" validation_identity ;;
    preflight)
        admit_source
        [[ "$(group_metadata version)" == "${RELEASE_PREVIOUS:?}" ]]
        ledger="$(group_metadata plan | jq -r .changelog)"
        awk -v version="${RELEASE_VERSION:?}" -v previous="${RELEASE_PREVIOUS:?}" -v date="${RELEASE_DATE:?}" \
            -f "$shared/finalize-release-changelog.awk" "$ledger" >/dev/null
        "$make_bin" --no-print-directory tools-check toko-tooling-check
        editor --check >/dev/null
        ;;
    verify)
        bash "$shared/check-release-source.sh"
        mkdir -p "$state"
        mkdir -p target/validation
        attempt=''
        if [[ -f "$selection" && ! -L "$selection" ]]; then
            IFS= read -r attempt < "$selection"
            [[ "$attempt" =~ ^target/validation/release\.[A-Za-z0-9]+$ ]] || exit 1
            prior_source="$(jq -er '.source' "$attempt/plan.json")"
            if [[ "$prior_source" != "$RELEASE_SOURCE" ]]; then
                echo 'Validation source changed; retaining the prior attempt and selecting a fresh gate.' >&2
                attempt=''
            fi
        fi
        if [[ -z "$attempt" || ! -f "$attempt/completed" ]]; then
            release_receipt_archive "$state/selection-archive.XXXXXX" "$selection" "$validation"
            attempt="$(mktemp -d target/validation/release.XXXXXX)"
            perl "$shared/release-timing.pl" run query-candidate-prepare executed bash scripts/ci/reuse-validation.sh prepare "$attempt"
            perl "$shared/release-timing.pl" run query-plan-selection executed bash scripts/ci/validation.sh plan "$attempt"
            release_receipt_write "$selection" printf '%s\n' "$attempt"
            QUERY_VALIDATION_ATTEMPT="$root/$attempt" \
                VALIDATION_REPOSITORY_ROOT="$root" VALIDATION_LOG_DIR="$attempt/logs" \
                VALIDATION_FAILURE_LOG_DIR="$state/validation-failures" \
                bash "$shared/run-validation-targets.sh" --plan "$attempt/plan.json"
            release_receipt_write "$attempt/completed" printf '%s\n' "$RELEASE_SOURCE"
        fi
        release_timing_begin query-plan-finalization
        jq -e --arg host "$(uname -sm)" --arg rust "$(rustc -Vv)" --arg cargo "$(cargo -V)" \
            '.host == $host and .rust == $rust and .cargo == $cargo' "$attempt/context.json" >/dev/null
        runs=("$attempt"/logs/validation.*)
        [[ ${#runs[@]} == 1 && -d "${runs[0]}" ]] || exit 1
        if [[ -e "$attempt/proof.json" || -L "$attempt/proof.json" ]]; then
            perl "$shared/validation-evidence.pl" check "$root" "$attempt/plan.json" \
                "$RELEASE_SOURCE" "$group" "${runs[0]}" "$attempt/context.json" "$attempt/proof.json" >/dev/null
        else
            release_receipt_write "$attempt/proof.json" perl "$shared/validation-evidence.pl" verify \
                "$root" "$attempt/plan.json" "$RELEASE_SOURCE" "$group" "${runs[0]}" "$attempt/context.json"
        fi
        release_timing_end 0
        release_timing_begin query-receipt-finalization
        bash "$shared/check-release-source.sh"
        release_receipt_write "$validation" validation_identity
        release_timing_end 0
        ;;
    prepare)
        release_receipt_check "$validation" validation_identity
        CARGO_RELEASE_METADATA_EDITOR="$(editor --check)"
        export CARGO_RELEASE_METADATA_EDITOR
        export CARGO_RELEASE_METADATA_MAX_BYTES=8388608
        group_metadata prepare
        if [[ "$group" == cli ]]; then
            perl -pi -e 's/^## CLI \Q$ENV{RELEASE_VERSION}\E — Unreleased$/## CLI $ENV{RELEASE_VERSION} — $ENV{RELEASE_DATE}/; s/^After publication, with /With /' CHANGELOG.md
            perl -pi -e 's/^After publication, install /Install /' releases/cli/CHANGELOG.md
        fi
        group_metadata check
        release_receipt_write "$payload" payload_identity
        ;;
    prepared|commit|committed)
        admit_source
        group_metadata check
        release_receipt_check "$validation" validation_identity
        release_receipt_check "$payload" payload_identity
        if [[ "$1" == commit ]]; then git diff --quiet; fi
        ;;
    publication)
        bash "$shared/check-release-source.sh"
        "$make_bin" --no-print-directory toko-tooling-check
        version="$(group_metadata version)"
        tag="$group-v$version"
        [[ "$(git cat-file -t "refs/tags/$tag")" == tag ]]
        [[ "$(git rev-parse "$tag^{commit}")" == "$(git rev-parse HEAD)" ]]
        [[ -z "${RELEASE_COMMIT:-}" || "$RELEASE_COMMIT" == "$(git rev-parse HEAD)" ]]
        ;;
    *) exit 2 ;;
esac
