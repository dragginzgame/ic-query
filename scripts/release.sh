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
validation="$state/$group-${RELEASE_VERSION:-unset}.query-validated"
payload="$state/$group-${RELEASE_VERSION:-unset}.query-payload"
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
        if [[ "${RELEASE_SETUP_MODE:-check}" == prepare ]]; then
            cargo fetch --locked
            "$make_bin" --no-print-directory install-tools
            editor >/dev/null
        else
            cargo fetch --locked --offline
        fi
        "$make_bin" --no-print-directory tools-check toko-tooling-check
        editor --check >/dev/null
        ;;
    version) group_metadata version ;;
    files) group_metadata files ;;
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
        VALIDATION_REPOSITORY_ROOT="$root" VALIDATION_FAILURE_LOG_DIR="$state/validation-failures" \
            bash "$shared/run-validation-targets.sh" --fail-fast check
        release_receipt_write "$validation" release_receipt_identity
        ;;
    prepare)
        release_receipt_check "$validation" release_receipt_identity
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
        release_receipt_check "$validation" release_receipt_identity
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
