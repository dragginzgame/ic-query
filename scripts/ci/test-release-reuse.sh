#!/usr/bin/env bash
# Explicit Linux qualification: real Cargo and canonical metadata preparation;
# commits are disposable fixture state, and no upload/tag/push is performed.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
[[ "$(uname -s)" == Linux ]] || { echo 'Native reuse qualification requires Linux.' >&2; exit 2; }
export PATH="$root/.tools/host/bin:$root/.tools/rust/common/bin:$PATH"
mkdir -p target/reuse-qualification
fixture="$(mktemp -d "$root/target/reuse-qualification/sequence.XXXXXX")"
printf 'Native sequence retained: %s\n' "$fixture"
current_log=''
on_exit() {
    local status=$?
    if [[ "$status" != 0 && -n "$current_log" && -f "$current_log" ]]; then
        printf 'Qualification failed; last bounded excerpt from %s:\n' "$current_log" >&2
        tail -80 "$current_log" >&2 || :
    fi
    exit "$status"
}
trap on_exit EXIT
git ls-files --cached --others --exclude-standard -z > "$fixture/files"
tar -c --null -T "$fixture/files" | tar -x -C "$fixture"
rm "$fixture/files"
cd "$fixture"
export QUERY_VALIDATION_TOOLS_ROOT="$root"
export CARGO_RELEASE_METADATA_EDITOR
CARGO_RELEASE_METADATA_EDITOR="$(bash vendor/toko-tooling/scripts/dev/install-rust-tools.sh --consumer "$root" \
    --package ic-build --version 0.73.0 --bin ic-build-release-metadata --profile debug --features cli --check)"
export CARGO_RELEASE_METADATA_MAX_BYTES=8388608
shared=vendor/toko-tooling/scripts/ci
# Tool payloads remain read-only in the explicitly selected caller. Every other
# common check, product command, selector, proof and metadata transaction is real.
cat >> Makefile <<'MAKE'
tools-check:
	+@$(MAKE) --no-print-directory -C "$(QUERY_VALIDATION_TOOLS_ROOT)" tools-check
MAKE
git init -q
git config user.name 'Query qualification'
git config user.email 'qualification@example.invalid'
git config core.hooksPath /dev/null
git config commit.gpgsign false
git remote add origin https://example.invalid/query
mkdir -p target/sequence
groups=(topology sns-migration query cli)
for group in "${groups[@]}"; do
    previous="$(bash "$shared/cargo-release-group.sh" version "$group")"
    IFS=. read -r major minor patch <<< "$previous"
    next="$major.$minor.$((patch + 1))"
    # Fixture-only draft headings use the canonical editor for all Cargo edits.
    ledger="$(jq -r --arg group "$group" '.groups[] | select(.name == $group) | .changelog' ci/cargo-release-groups.json)"
    printf '# Qualification ledger\n\n## [%s] - Unreleased\n\nFixture release.\n' "$next" > "$ledger"
    printf '%s\t%s\t%s\n' "$group" "$previous" "$next" >> target/sequence/versions.tsv
done
git add .
git commit -qm 'Native sequence qualification baseline'
while IFS=$'\t' read -r group previous _; do git tag -a "$group-v$previous" -m 'Fixture baseline'; done < target/sequence/versions.tsv
# A shared change models adoption of this release wiring: the first group must
# qualify all product boundaries, and the later groups must reuse that proof.
printf '\n' >> Cargo.lock
git add Cargo.lock
git commit -qm 'Shared release input change'
export RELEASE_REMOTE=origin RELEASE_BRANCH=main RELEASE_DATE
RELEASE_DATE="$(date -u +%F)"
first_identity=''
printf 'group\tseconds\tstage\tmode\n' > target/sequence/timings.tsv
while IFS=$'\t' read -r group previous next; do
    export RELEASE_GROUP="$group" RELEASE_PREVIOUS="$previous" RELEASE_VERSION="$next" RELEASE_SOURCE
    RELEASE_SOURCE="$(git rev-parse HEAD)"
    started=$SECONDS
    current_log="target/sequence/$group.log"
    bash scripts/release.sh verify > "$current_log" 2>&1
    seconds=$((SECONDS - started))
    selection=".git/release-state/$group-$next.query-selection"
    IFS= read -r attempt < "$selection"
    identity="$(jq -er .identity "$attempt/projection.json")"
    if [[ -z "$first_identity" ]]; then first_identity="$identity"; else [[ "$identity" == "$first_identity" ]]; fi
    seal="$attempt/products.seal.json"
    stage="$(jq -er .admission.stage "$seal")"
    mode="$(jq -er .admission.mode "$seal")"
    if [[ "$group" == topology ]]; then
        [[ "$mode" == executed ]]
    else
        [[ "$mode" == reused ]]
    fi
    printf '%s\t%s\t%s\t%s\n' "$group" "$seconds" "$stage" "$mode" >> target/sequence/timings.tsv
    # Every group freshly checks the actual intermediate CLI version/help.
    rg -q 'test result: ok. 28 passed' "$attempt"/logs/validation.*/*.log
    bash "$shared/cargo-release-group.sh" prepare "$group" >> "target/sequence/$group.log" 2>&1
    bash scripts/release.sh validated
    git add .
    git commit -qm "Simulate $group metadata delivery"
    printf 'Qualified %s: %ss; canonical candidate identity unchanged.\n' "$group" "$seconds"
done < target/sequence/versions.tsv
[[ -z "$(git status --porcelain)" ]]
cat target/sequence/timings.tsv
next="$(awk '$1 == "cli" {print $3}' target/sequence/versions.tsv)"
export RELEASE_GROUP=cli RELEASE_VERSION="$next"
RELEASE_SOURCE="$(git rev-parse HEAD)"
export RELEASE_PREVIOUS="$next"
git tag -a "cli-v$next" -m 'Fixture delivered baseline'
prepare_attempt() {
    mkdir -p "target/sequence/$1"
    current_log="target/sequence/$1/prepare.log"
    bash scripts/ci/reuse-validation.sh prepare "target/sequence/$1" > "$current_log" 2>&1
    bash scripts/ci/validation.sh plan "target/sequence/$1" >> "$current_log" 2>&1
    export QUERY_VALIDATION_ATTEMPT="$fixture/target/sequence/$1"
}
run_selected() {
    current_log="$QUERY_VALIDATION_ATTEMPT/stage.log"
    bash scripts/ci/reuse-validation.sh stage validate-cli > "$current_log" 2>&1
}
prepare_attempt seal-failure
run_selected
jq -e '(.command[0] | endswith("/make")) and .command[1:] == ["--no-print-directory","validate-cli"]' "$QUERY_VALIDATION_ATTEMPT/products.seal.json" >/dev/null
echo 'Qualified an isolated CLI selection with real Cargo and no specialist test dispatch.'
mv "$QUERY_VALIDATION_ATTEMPT/products.seal.json" target/sequence/original-seal.json
mkdir "$QUERY_VALIDATION_ATTEMPT/products.seal.json"
# Inject failure in the exact atomic writer used by the adapter, after real
# execution. A new attempt must recover from the original execution proof.
source "$shared/release-receipts.sh"
if release_receipt_write "$QUERY_VALIDATION_ATTEMPT/products.seal.json" perl "$shared/seal-validation-stage.pl" \
    "$QUERY_VALIDATION_ATTEMPT/products.admission.json" "$RELEASE_SOURCE" "$RELEASE_GROUP" \
    "$(jq -er .identity "$QUERY_VALIDATION_ATTEMPT/projection.json")"; then
    echo 'seal output failure accepted' >&2; exit 1
fi
original_proof="$(jq -er .execution_proof "$QUERY_VALIDATION_ATTEMPT/products.admission.json")"
prepare_attempt seal-recovery
run_selected
jq -e --arg proof "$original_proof" '.mode == "reused" and .execution_proof == $proof' \
    "$QUERY_VALIDATION_ATTEMPT/products.admission.json" >/dev/null
echo 'Qualified seal-output failure recovery without repeating Cargo.'

original_log="$(jq -er .logs.stdout.path "$original_proof")"
cp "$original_log" target/sequence/original-log
printf 'tamper\n' >> "$original_log"
prepare_attempt corrupt-proof
run_selected
jq -e '.mode == "executed"' "$QUERY_VALIDATION_ATTEMPT/products.admission.json" >/dev/null
cp target/sequence/original-log "$original_log"
echo 'Qualified corrupted execution evidence: retained diagnosis and fresh execution.'

prepare_attempt environment-change
jq -cS '.TZ = "Europe/Monaco"' "$QUERY_VALIDATION_ATTEMPT/reuse-environment.json" > target/sequence/environment.json
mv target/sequence/environment.json "$QUERY_VALIDATION_ATTEMPT/reuse-environment.json"
run_selected
jq -e '.mode == "executed"' "$QUERY_VALIDATION_ATTEMPT/products.admission.json" >/dev/null
echo 'Qualified bounded-environment invalidation.'

good_source="$RELEASE_SOURCE"
printf '\ncompile_error!("qualification failure");\n' >> crates/ic-query-cli/src/main.rs
git add crates/ic-query-cli/src/main.rs
git commit -qm 'Inject required product failure'
RELEASE_SOURCE="$(git rev-parse HEAD)"
prepare_attempt product-failure
if run_selected; then echo 'required product failure accepted' >&2; exit 1; fi
[[ ! -e "$QUERY_VALIDATION_ATTEMPT/products.admission.json" ]]
git checkout "$good_source" -- crates/ic-query-cli/src/main.rs
git commit -qm 'Restore qualified product input'
RELEASE_SOURCE="$(git rev-parse HEAD)"
prepare_attempt product-recovery
run_selected
jq -e '.mode == "reused"' "$QUERY_VALIDATION_ATTEMPT/products.admission.json" >/dev/null
echo 'Qualified failed-product refusal and recovery to original exact inputs.'
unset QUERY_VALIDATION_ATTEMPT
RELEASE_PREVIOUS="$(awk '$1 == "cli" {print $2}' target/sequence/versions.tsv)"
started=$SECONDS
current_log=target/sequence/warm-full.log
make --no-print-directory check > "$current_log" 2>&1
printf 'Warm dependency cache, ordinary full gate: %ss\n' "$((SECONDS - started))"
started=$SECONDS
current_log=target/sequence/warm-reused.log
bash scripts/release.sh verify > "$current_log" 2>&1
seconds=$((SECONDS - started))
attempt="$(cat ".git/release-state/cli-$RELEASE_VERSION.query-selection")"
mode="$(jq -er .mode "$attempt/products.admission.json")"
printf 'Fresh complete release admission: %ss; product checks %s\n' "$seconds" "$mode"
printf 'warm-admission\t%s\tquery-products\t%s\n' "$seconds" "$mode" >> target/sequence/timings.tsv
echo 'Four native groups qualified; all metadata transactions local, no registry/network delivery.'
