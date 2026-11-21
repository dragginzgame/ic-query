#!/usr/bin/env bash
# Explicit Linux qualification: real Cargo and canonical metadata preparation;
# commits are disposable fixture state, and no upload/tag/push is performed.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
baseline=''
if [[ $# != 0 ]]; then
    [[ $# == 2 && "$1" == --baseline ]] || { echo 'usage: test-release-reuse.sh [--baseline COMMIT]' >&2; exit 2; }
    baseline="$(git rev-parse --verify "$2^{commit}")"
fi
[[ "$(uname -s)" == Linux ]] || { echo 'Native reuse qualification requires Linux.' >&2; exit 2; }
export PATH="$root/.tools/host/bin:$root/.tools/rust/common/bin:$PATH"
mkdir -p target/reuse-qualification
fixture="$(mktemp -d "$root/target/reuse-qualification/sequence.XXXXXX")"
printf 'Native sequence retained: %s\n' "$fixture"
current_log=''
on_exit() {
    local status=$?
    if [[ -n "${shared:-}" && -s "${RELEASE_TIMING_JOURNAL:-}" ]]; then
        # Retain bounded timing fields, not environment inventories or tool paths.
        if ! perl "$shared/release-timing.pl" report "$RELEASE_TIMING_JOURNAL" \
            | jq -r '.[] | [.group,.phase,.mode,.result,.elapsed_seconds] | @tsv' \
                > "${RELEASE_TIMING_JOURNAL%.jsonl}.tsv"; then
            echo 'Could not summarize qualification phase timings.' >&2
            [[ "$status" != 0 ]] || status=1
        fi
    fi
    if [[ "$status" != 0 && -n "$current_log" && -f "$current_log" ]]; then
        printf 'Qualification failed; last bounded excerpt from %s:\n' "$current_log" >&2
        tail -80 "$current_log" >&2 || :
    fi
    exit "$status"
}
trap on_exit EXIT
if [[ -n "$baseline" ]]; then
    git archive "$baseline" | tar -x -C "$fixture"
else
    git ls-files --cached --others --exclude-standard -z > "$fixture/files"
    tar -c --null -T "$fixture/files" | tar -x -C "$fixture"
    rm "$fixture/files"
fi
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
export RELEASE_TIMING_JOURNAL="$fixture/target/sequence/phases.jsonl"
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
    if [[ -z "$baseline" ]]; then
        jq -e '[.stages[] | select(.required) | .target] == ["validate-common","validate-products","validate-current-cli"]' "$attempt/plan.json" >/dev/null
        # One semantic product admission and one seal per group, regardless of
        # how many product obligations the original map selects.
        perl "$shared/release-timing.pl" report "$RELEASE_TIMING_JOURNAL" > target/sequence/phases.json
        jq -e --arg group "$group" '[.[] | select(.group == $group and .phase == "query-product-seal-write")] | length == 1' target/sequence/phases.json >/dev/null
        jq -e --arg group "$group" '[.[] | select(.group == $group and .phase == "query-product-seal-check")] | length == 0' target/sequence/phases.json >/dev/null
    fi
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
    if [[ -z "$baseline" && "$group" == topology ]]; then
        for evidence in obligations.json execution-policy.json reuse-environment.json; do
            cp "$attempt/$evidence" target/sequence/saved-input
            printf '\n' >> "$attempt/$evidence"
            if bash scripts/release.sh validated; then echo 'changed plan input accepted' >&2; exit 1; fi
            cp target/sequence/saved-input "$attempt/$evidence"
        done
        bash scripts/release.sh validated
    fi
    bash "$shared/cargo-release-group.sh" prepare "$group" >> "target/sequence/$group.log" 2>&1
    bash scripts/release.sh validated
    git add .
    git commit -qm "Simulate $group metadata delivery"
    printf 'Qualified %s: %ss; canonical candidate identity unchanged.\n' "$group" "$seconds"
done < target/sequence/versions.tsv
[[ -z "$(git status --porcelain)" ]]
cat target/sequence/timings.tsv
if [[ -n "$baseline" ]]; then
    printf 'Baseline four-group sequence complete at %s; simulated metadata only.\n' "$baseline"
    exit 0
fi
current_log=target/sequence/setup.log
# Prepare the fixture's selected editor once; subsequent setup must admit it
# without invoking installation again. Real dependencies remain locked; Cargo settings are inherited.
RELEASE_SOURCE="$(git rev-parse HEAD)"
RELEASE_SETUP_MODE=prepare bash scripts/release.sh setup > "$current_log" 2>&1
cat > target/sequence/setup-make <<'MAKE'
#!/usr/bin/env bash
set -euo pipefail
case "${*: -1}" in
    tools-check) [[ -f "$SETUP_READY" ]] ;;
    install-tools) echo install >> "$SETUP_TRACE"; touch "$SETUP_READY" ;;
    toko-tooling-check) exec make --no-print-directory toko-tooling-check ;;
    *) exit 2 ;;
esac
MAKE
chmod +x target/sequence/setup-make
export RELEASE_MAKE="$fixture/target/sequence/setup-make"
export SETUP_READY="$fixture/target/sequence/setup-ready" SETUP_TRACE="$fixture/target/sequence/setup-trace"
touch "$SETUP_READY" "$SETUP_TRACE"
RELEASE_SETUP_MODE=prepare bash scripts/release.sh setup >> "$current_log" 2>&1
[[ ! -s "$SETUP_TRACE" ]]
rm "$SETUP_READY"
if RELEASE_SETUP_MODE=check bash scripts/release.sh setup >> "$current_log" 2>&1; then echo 'recovery installed missing tools' >&2; exit 1; fi
[[ ! -s "$SETUP_TRACE" ]]
RELEASE_SETUP_MODE=prepare bash scripts/release.sh setup >> "$current_log" 2>&1
[[ "$(cat "$SETUP_TRACE")" == install ]]
unset RELEASE_MAKE SETUP_READY SETUP_TRACE
echo 'Qualified prepared-tool admission, missing-tool preparation and check-only recovery (installation effects simulated).'
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
    bash scripts/ci/reuse-validation.sh products > "$current_log" 2>&1
}
prepare_attempt seal-failure
jq -e --arg selected "${CARGO_NET_OFFLINE-unset}" \
    'if $selected == "unset" then has("CARGO_NET_OFFLINE") | not
     else .CARGO_NET_OFFLINE == $selected end' \
    "$QUERY_VALIDATION_ATTEMPT/reuse-environment.json" >/dev/null
environment="$QUERY_VALIDATION_ATTEMPT/reuse-environment.json"
cp "$environment" target/sequence/caller-environment.json
unexpected_mode=true
[[ "${CARGO_NET_OFFLINE-unset}" != true ]] || unexpected_mode=false
jq -cS --arg mode "$unexpected_mode" '.CARGO_NET_OFFLINE = $mode' \
    target/sequence/caller-environment.json > "$environment"
if run_selected; then echo 'stage-selected Cargo network mode accepted' >&2; exit 1; fi
grep -q 'conflicting invocation and stage CARGO_NET_OFFLINE settings' "$current_log"
[[ ! -e "$QUERY_VALIDATION_ATTEMPT/products.admission.json" ]]
mv "$current_log" target/sequence/network-mode-refusal.log
cp target/sequence/caller-environment.json "$environment"
echo 'Qualified refusal of a stage-selected Cargo network mode before product admission.'
run_selected
jq -e '(.command[0] | endswith("/make")) and .command[1:] == ["--no-print-directory","validate-cli"]' "$QUERY_VALIDATION_ATTEMPT/products.seal.json" >/dev/null
echo 'Qualified an isolated CLI selection with real Cargo and no specialist test dispatch.'
# An already sealed admission must refuse damaged original evidence, rather
# than accepting the presence of its output file. Restoration permits recovery.
proof="$(jq -er .execution_proof "$QUERY_VALIDATION_ATTEMPT/products.admission.json")"
log="$(jq -er .logs.stdout.path "$proof")"
for evidence in "$proof" "$log" "$QUERY_VALIDATION_ATTEMPT/products.admission.json"; do
    cp "$evidence" target/sequence/saved-evidence
    printf 'tamper\n' >> "$evidence"
    if run_selected; then echo 'changed sealed evidence accepted' >&2; exit 1; fi
    cp target/sequence/saved-evidence "$evidence"
    mv "$evidence" target/sequence/missing-evidence
    if run_selected; then echo 'missing sealed evidence accepted' >&2; exit 1; fi
    mv target/sequence/missing-evidence "$evidence"
done
run_selected
echo 'Qualified missing/tampered proof, log and admission refusal on sealed recovery.'
cp "$QUERY_VALIDATION_ATTEMPT/obligations.json" target/sequence/saved-obligations
jq -cS '(.stages[] | select(.target == "validate-topology")).required = true' \
    target/sequence/saved-obligations > "$QUERY_VALIDATION_ATTEMPT/obligations.json"
if run_selected; then echo 'changed product obligations accepted' >&2; exit 1; fi
cp target/sequence/saved-obligations "$QUERY_VALIDATION_ATTEMPT/obligations.json"
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
# Re-admit the full selection after the isolated failure/recovery cases. The
# pinned four-group baseline above owns the comparable timing measurement;
# repeating an ordinary full gate here supplies no additional release guarantee.
started=$SECONDS
current_log=target/sequence/warm-reused.log
bash scripts/release.sh verify > "$current_log" 2>&1
seconds=$((SECONDS - started))
attempt="$(cat ".git/release-state/cli-$RELEASE_VERSION.query-selection")"
mode="$(jq -er .mode "$attempt/products.admission.json")"
printf 'Fresh complete release admission: %ss; product checks %s\n' "$seconds" "$mode"
printf 'warm-admission\t%s\tquery-products\t%s\n' "$seconds" "$mode" >> target/sequence/timings.tsv
echo 'Four native groups qualified; all metadata transactions local, no registry/network delivery.'
