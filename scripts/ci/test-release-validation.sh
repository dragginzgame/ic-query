#!/usr/bin/env bash
# Consumer wiring fixtures: real Query recipes/selector/shared runner, fake Cargo.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
export PATH="$root/.tools/host/bin:$root/.tools/rust/common/bin:$PATH"
mkdir -p "$root/target/validation-fixtures"
fixture="$(mktemp -d "$root/target/validation-fixtures/query.XXXXXX")"
echo "Retained release fixture: $fixture"
git ls-files --cached --others --exclude-standard -z > "$fixture/files"
tar -c --null -T "$fixture/files" | tar -x -C "$fixture"
cd "$fixture"
rm files
cat >> Makefile <<'MAKE'
# These fixtures test dispatch, not tool installation or native compilation.
toko-tooling-check tools-check doc-links-check fmt-check format-tools-check:
	@printf 'common %s\n' '$@' >> "$$TRACE"
MAKE
mkdir -p .tools/host/bin target
cat > .tools/host/bin/cargo <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo' >> "$TRACE"
printf ' %s' "$@" >> "$TRACE"
printf '\n' >> "$TRACE"
[[ "${1:-}" != -V ]] || { echo 'fixture cargo'; exit 0; }
if [[ -n "${FAIL_PACKAGE:-}" && " $* " == *" -p $FAIL_PACKAGE "* ]]; then exit 19; fi
CARGO
cat > .tools/host/bin/rustc <<'RUSTC'
#!/usr/bin/env bash
echo 'fixture rustc'
RUSTC
chmod +x .tools/host/bin/cargo .tools/host/bin/rustc
git init -q
git config user.name 'Query fixture'
git config user.email 'fixture@example.invalid'
git config core.hooksPath /dev/null
git config commit.gpgsign false
git remote add origin https://example.invalid/query
git add .
git commit -qm 'Fixture baseline'
base="$(git rev-parse HEAD)"
for group in topology sns-migration query cli; do git tag -a "$group-v0.0.1" -m fixture; done
export TRACE="$fixture/target/trace"
export RELEASE_SOURCE="$base" RELEASE_PREVIOUS=0.0.1 RELEASE_VERSION=0.0.2
export RELEASE_DATE=2026-11-17 RELEASE_REMOTE=origin RELEASE_BRANCH=main
shared=vendor/toko-tooling/scripts/ci
assertions=0
ok() { assertions=$((assertions + 1)); printf 'ok %s - %s\n' "$assertions" "$1"; }
select_plan() {
    export RELEASE_GROUP="$1"
    mkdir -p "target/$2"
    bash scripts/ci/validation.sh plan "target/$2" >"target/$2/select.log" 2>&1
}
expect_targets() {
    local observed
    observed="$(jq -r '[.stages[] | select(.required) | .target] | join(" ")' "target/$1/plan.json")"
    [[ "$observed" == "$2" ]] || { echo "unexpected stages: $observed" >&2; exit 1; }
    ok "$1 mapping"
}
run_plan() {
    : > "$TRACE"
    VALIDATION_REPOSITORY_ROOT="$fixture" VALIDATION_LOG_DIR="target/$1/logs" \
        bash "$shared/run-validation-targets.sh" --plan "target/$1/plan.json" >"target/$1/dispatch.log" 2>&1
}
for group in cli query topology sns-migration; do
    select_plan "$group" "$group"
    case "$group" in
        cli) expected='validate-common validate-cli' ;;
        query) expected='validate-common qualify-umbrella' ;;
        topology) expected='validate-common validate-topology validate-cli qualify-umbrella' ;;
        sns-migration) expected='validate-common validate-sns validate-cli qualify-umbrella' ;;
    esac
    expect_targets "$group" "$expected"
    run_plan "$group"
    if rg -q -- '--workspace' "$TRACE"; then exit 1; fi
    if [[ "$group" == cli ]]; then
        if rg -q -- '-p ic-query-topology|-p ic-query-sns|-p ic-query ' "$TRACE"; then exit 1; fi
        rg -q '^cargo test -p ic-query-cli --locked --offline$' "$TRACE"
    elif [[ "$group" == topology ]]; then
        if rg -q -- '-p ic-query-sns' "$TRACE"; then exit 1; fi
        rg -q '^cargo test -p ic-query-topology --locked --offline$' "$TRACE"
        rg -q '^cargo test -p ic-query-cli --locked --offline$' "$TRACE"
        rg -q '^cargo test -p ic-query --all-features --locked --offline$' "$TRACE"
    elif [[ "$group" == sns-migration ]]; then
        if rg -q -- '-p ic-query-topology' "$TRACE"; then exit 1; fi
    fi
    ok "$group actual Cargo leaves and unrelated-package omission"
done
[[ -z "$(git status --porcelain)" ]]
ok 'sequential group selection preserves all versions and ledgers'
# A specialist implementation change broadens even a selected CLI release.
printf '\n// fixture change\n' >> crates/ic-query-topology/src/lib.rs
git add crates/ic-query-topology/src/lib.rs
git commit -qm 'Specialist change'
RELEASE_SOURCE="$(git rev-parse HEAD)"
select_plan cli changed-specialist
expect_targets changed-specialist 'validate-common validate-topology validate-cli qualify-umbrella'
# Locks and unknown paths must conservatively select the full roster.
printf '\n' >> Cargo.lock
git add Cargo.lock
git commit -qm 'Shared change'
RELEASE_SOURCE="$(git rev-parse HEAD)"
select_plan cli shared
expect_targets shared 'validate-common validate-topology validate-sns validate-cli qualify-umbrella'
git reset --hard -q "$base"
export RELEASE_SOURCE="$base"
printf 'unknown input\n' > new-input
git add new-input
git commit -qm 'Unknown input'
RELEASE_SOURCE="$(git rev-parse HEAD)"
select_plan cli unknown-input
expect_targets unknown-input 'validate-common validate-topology validate-sns validate-cli qualify-umbrella'
git reset --hard -q "$base"
export RELEASE_SOURCE="$base"
RELEASE_PREVIOUS=9.9.9 select_plan cli missing-baseline
expect_targets missing-baseline 'validate-common validate-topology validate-sns validate-cli qualify-umbrella'
if select_plan unknown unknown; then echo 'unknown group accepted' >&2; exit 1; fi
ok 'unknown group refuses'
cp ci/release-validation.json target/original-policy
jq -cS '.stages += ["unmapped-stage"]' target/original-policy > ci/release-validation.json
if select_plan cli extra-stage; then echo 'unmapped stage accepted' >&2; exit 1; fi
cp target/original-policy ci/release-validation.json
ok 'new roster obligation requires a mapping'
select_plan cli excluded-failure
FAIL_PACKAGE=ic-query-sns run_plan excluded-failure
ok 'excluded specialist failure does not block CLI'
select_plan cli required-failure
if FAIL_PACKAGE=ic-query-cli run_plan required-failure; then echo 'required failure accepted' >&2; exit 1; fi
ok 'required CLI failure blocks'
select_plan cli stale-plan
printf '\n// source drift\n' >> crates/ic-query-cli/src/main.rs
git add crates/ic-query-cli/src/main.rs
git commit -qm 'Source drift'
if run_plan stale-plan; then echo 'stale plan accepted' >&2; exit 1; fi
ok 'source drift refuses stale plan'
git reset --hard -q "$base"

# Exercise the actual release hook and its protected receipt/recovery, not a
# second implementation of event parsing. Cargo effects remain simulated.
export RELEASE_GROUP=cli
# This fixture qualifies ordinary dispatch and same-source recovery. Native
# candidate reuse has its own real-Cargo qualification; an override must fall back.
export RUSTFLAGS=fixture
: > "$TRACE"
bash scripts/release.sh verify >target/release.log 2>&1
selection=.git/release-state/cli-0.0.2.query-selection
IFS= read -r attempt < "$selection"
[[ -s "$attempt/proof.json" ]]
ok 'release hook writes admitted evidence'
bash scripts/release.sh validated
printf '\n' >> Cargo.lock
bash scripts/release.sh validated
git checkout -- Cargo.lock
ok 'original validation custody survives separately admitted metadata changes'
cp "$attempt/context.json" target/original-context
printf 'tampered\n' >> "$attempt/context.json"
if bash scripts/release.sh validated; then echo 'changed receipt context accepted' >&2; exit 1; fi
cp target/original-context "$attempt/context.json"
ok 'receipt refuses changed context after preparation'
cp "$TRACE" target/original-trace
bash scripts/release.sh verify >>target/release.log 2>&1
# Version observations are fresh on recovery; no product checks repeat.
rg -v '^cargo -V$' "$TRACE" > target/recovered-trace
rg -v '^cargo -V$' target/original-trace > target/original-checks
cmp target/recovered-trace target/original-checks
ok 'same-source recovery checks evidence without repeating Cargo'
cp "$attempt/proof.json" target/original-proof
rm "$attempt/proof.json"
bash scripts/release.sh verify >>target/release.log 2>&1
rg -v '^cargo -V$' "$TRACE" > target/recovered-trace
cmp target/recovered-trace target/original-checks
cmp "$attempt/proof.json" target/original-proof
ok 'post-execution proof-publication recovery does not repeat Cargo'
logs=("$attempt"/logs/validation.*)
cp "${logs[0]}/0.log" target/original-log
rm "${logs[0]}/0.log"
if bash scripts/release.sh verify >>target/release.log 2>&1; then echo 'missing log accepted' >&2; exit 1; fi
ok 'missing retained log refuses'
cp target/original-log "${logs[0]}/0.log"
printf 'tampered\n' >> "${logs[0]}/0.log"
if bash scripts/release.sh verify >>target/release.log 2>&1; then echo 'tampered evidence accepted' >&2; exit 1; fi
rg -v '^cargo -V$' "$TRACE" > target/recovered-trace
cmp target/recovered-trace target/original-checks
ok 'tampered retained log refuses without overwriting evidence'
cp target/original-log "${logs[0]}/0.log"
printf '\n// new release source\n' >> crates/ic-query-cli/src/main.rs
git add crates/ic-query-cli/src/main.rs
git commit -qm 'New release source'
RELEASE_SOURCE="$(git rev-parse HEAD)"
bash scripts/release.sh verify >>target/release.log 2>&1
IFS= read -r next_attempt < "$selection"
[[ "$next_attempt" != "$attempt" && -f "$attempt/proof.json" && -f "$next_attempt/proof.json" ]]
ok 'new source executes a fresh gate and preserves prior evidence'
printf 'Passed %s Query release wiring assertions (simulated Cargo; no publication).\n' "$assertions"
