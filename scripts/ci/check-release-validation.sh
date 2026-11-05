#!/usr/bin/env bash
# Shared companions: scripts/ci/prepare-script-fixture.sh scripts/release/validation.sh
set -euo pipefail
case "${IC_QUERY_RELEASE_VALIDATION_CASE:-all}" in
  all|utf8) ;;
  *) echo 'unknown Query release-validation fixture case' >&2; exit 2 ;;
esac
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS
unset IC_TOOL_PINS IC_TOOLS FORMAT_CARGO SHARED_TOOLING_ROOT
repo_root="$(CDPATH='' cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
mkdir -p "${TMPDIR:-$repo_root/target/tmp}"
work_dir="$(mktemp -d "${TMPDIR:-$repo_root/target/tmp}/ic-query-group-validation.XXXXXX")"
work_dir="$(CDPATH='' cd -P -- "$work_dir" && pwd -P)"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "Group validation fixtures retained"' EXIT
export TMPDIR="${work_dir%/*}"
fail() { echo "Query group validation fixture failed: $*" >&2; exit 1; }
mkdir "$work_dir/repository"
bash "$repo_root/scripts/ci/prepare-script-fixture.sh" "$repo_root" "$work_dir/repository" \
  scripts/release/adapter.sh scripts/ci/run-ci.sh scripts/release/qualify-validation.sh
cd "$work_dir/repository"
git init -q
git config user.name 'Query validation fixture'
git config user.email 'query-validation@example.invalid'
git remote add origin https://example.invalid/query
printf 'target/\n' > .gitignore
printf '[workspace]\n' > Cargo.toml
mkdir -p crates/ic-query/src crates/ic-query-cli/src crates/ic-query-text/src crates/ic-query-testkit/src
printf 'baseline\n' > crates/ic-query/src/lib.rs
printf 'baseline\n' > crates/ic-query-cli/src/lib.rs
printf 'baseline\n' > crates/ic-query-text/src/lib.rs
printf 'baseline\n' > crates/ic-query-testkit/src/lib.rs
cat > scripts/ci/cargo-release-group.sh <<'STUB'
[[ "$1" == version && ( "$2" == query || "$2" == cli ) ]] || exit 2
printf '0.89.0\n'
STUB
cat > scripts/release/metadata.pl <<'STUB'
use strict;
use warnings;
my $operation = shift @ARGV;
if ($operation eq 'check-paths') {
    local $/ = "\0";
    while (my $path = <STDIN>) {
        chomp $path;
        $path eq 'Cargo.toml' or die "unrelated source changed: $path\n";
    }
} elsif ($operation ne 'prepare') { die "unexpected metadata operation\n"; }
STUB
cat > Makefile <<'MAKE'
.PHONY: $(shell jq -r '.stages[]' ci/release-validation.json)
$(shell jq -r '.stages[]' ci/release-validation.json):
	@printf '%s\n' '$@' >> target/trace
	@test '$@' != '$(FAIL_TARGET)'
	@if test '$@' = '$(INTERRUPT_TARGET)'; then kill -TERM $$$$; fi
	@if test '$@' = clippy && test -n '$(DRIFT_INPUT)'; then printf '\ninput drift\n' >> '$(DRIFT_INPUT)'; fi
	@if test '$@' = clippy; then bash scripts/ci/run-validation-targets.sh --fail-fast nested-check; fi
.PHONY: nested-check
nested-check:
	@printf 'retained nested output\n' > target/nested-output
MAKE
if [[ "${IC_QUERY_RELEASE_VALIDATION_CASE:-all}" == utf8 ]]; then
  jq -cS '.groups.cli.omit["query-test"] += " Query → CLI integration remains required."' \
    ci/release-validation-scopes.json > "$work_dir/scopes.json"
  cp "$work_dir/scopes.json" ci/release-validation-scopes.json
fi
git add --all
git commit -qm 'Validation fixture baseline'
export RELEASE_SOURCE RELEASE_GROUP=query RELEASE_PREVIOUS=0.89.0 RELEASE_VERSION=0.89.1
export RELEASE_DATE=2026-11-03 RELEASE_REMOTE=origin RELEASE_BRANCH=main RELEASE_KIND=patch
export CARGO_RELEASE_METADATA_EDITOR=/fixture/editor
RELEASE_SOURCE="$(git rev-parse HEAD)"
perl scripts/release/select-validation.pl context "$PWD" release > "$work_dir/context.json"
jq -cS '{schema:1,hosts:{(.host):.mapping}}' "$work_dir/context.json" > ci/release-validation-hosts.json
git add ci/release-validation-hosts.json
git commit -qm 'Qualify simulated native mapping'
baseline="$(git rev-parse HEAD)"
git tag -a query-v0.89.0 -m baseline
git tag -a cli-v0.89.0 -m baseline
run_gate() {
  local name="$1"
  mkdir -p "target/$name"
  : > target/trace
  RELEASE_SOURCE="$(git rev-parse HEAD)" bash scripts/release/validation.sh run "target/$name" > "$work_dir/$name.log" 2>&1
}
refuse() { if "$@"; then fail 'unexpected admission'; fi; }
if [[ "${IC_QUERY_RELEASE_VALIDATION_CASE:-all}" == utf8 ]]; then
  export RELEASE_GROUP=cli
  printf 'CLI owner change\n' >> crates/ic-query-cli/src/lib.rs
  git add --all
  git commit -qm 'CLI UTF-8 selection fixture'
  RELEASE_SOURCE="$(git rev-parse HEAD)"
  run_gate utf8
  gate="$(cat target/utf8/group-validation)"
  jq -e '.scope == "group" and any(.stages[];
    .target == "query-test" and (.required|not) and (.reason|contains("Query → CLI")))' \
    "$gate/plan.json" > /dev/null
  bash scripts/release/validation.sh check target/utf8
  printf 'Query UTF-8 selection, Shared evidence and retained receipt admission passed\n'
  fixture_complete=true
  exit 0
fi
for group in query cli; do
  git reset --hard -q "$baseline"
  export RELEASE_GROUP="$group"
  if [[ "$group" == query ]]; then owner=ic-query; else owner=ic-query-cli; fi
  printf 'owner change\n' >> "crates/$owner/src/lib.rs"
  git add --all
  git commit -qm "$group owner change"
  RELEASE_SOURCE="$(git rev-parse HEAD)"
  run_gate "$group"
  gate="$(cat "target/$group/group-validation")"
  jq -e --arg group "$group" '
    .scope == "group" and
    ([.stages[]|select(.required|not)|.target] ==
      (["ci-scripts-check","publish-guards-check","release-guards-check"] +
        if $group == "cli" then ["query-test"] else [] end)) and
    any(.stages[]; .target == "cli-test" and .required) and
    any(.stages[]; .target == "candid-boundary-check" and .required) and
    any(.stages[]; .target == "feature-boundary-check" and .required) and
    any(.stages[]; .target == "package" and .required)
  ' "$gate/plan.json" > /dev/null
  jq -r '.stages[] | select(.required) | .target' "$gate/plan.json" > "$work_dir/expected"
  cmp "$work_dir/expected" target/trace
  bash scripts/release/validation.sh check "target/$group"
  # Exercise the real adapter's saved-intent recovery admission. Metadata's
  # version-byte authentication is separately qualified by its existing suite.
  attempt=".git/release-state/$group-0.89.1.verify.fixture"
  mkdir -p "$attempt"
  cp "target/$group/group-validation" "$attempt/group-validation"
  printf '%s\n' validation-1 "$RELEASE_KIND" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" \
    "$RELEASE_DATE" "$RELEASE_SOURCE" "$RELEASE_REMOTE" "$RELEASE_BRANCH" "$RELEASE_GROUP" \
    "$(perl -MDigest::SHA=sha256_hex -0777 -ne 'print sha256_hex($_)' ci/cargo-release-groups.json)" > "$attempt/identity"
  : > "$attempt/passed"
  printf '%s\n' "$attempt" > ".git/release-state/$group-0.89.1.validation"
  bash scripts/release/adapter.sh prepare
  printf '# prepared version payload\n' >> Cargo.toml
  bash scripts/release/adapter.sh prepare
  printf 'unrelated source\n' >> "crates/$owner/src/lib.rs"
  refuse bash scripts/release/adapter.sh prepare > "$work_dir/source-drift.log" 2>&1
  git checkout -- Cargo.toml "crates/$owner/src/lib.rs"
  # Unrelated automation failures cannot block this qualified owner-only gate.
  FAIL_TARGET=release-guards-check run_gate "$group-excluded"
  FAIL_TARGET=clippy refuse run_gate "$group-required"
  [[ ! -e "target/$group-required/group-validation" ]] || fail 'failed stage produced successful evidence'
  FAIL_TARGET=cli-test refuse run_gate "$group-consumer-required"
  if [[ "$group" == cli ]]; then
    FAIL_TARGET=query-test run_gate cli-engine-excluded
  else
    FAIL_TARGET=query-test refuse run_gate query-engine-required
  fi
  INTERRUPT_TARGET=cli-test refuse run_gate "$group-interrupted"
  [[ ! -e "target/$group-interrupted/group-validation" ]] || fail 'interrupted stage produced successful evidence'
  # Recovery authenticates retained runner evidence, compiler context and baseline.
  printf 'changed original log\n' >> "$gate/$(jq -r '.run' "$gate/proof.json" | sed "s|^$gate/||")/0.log"
  refuse bash scripts/release/validation.sh check "target/$group" > "$work_dir/tampered.log" 2>&1
done
git reset --hard -q "$baseline"
export RELEASE_GROUP=cli
printf 'presentation change\n' >> crates/ic-query-text/src/lib.rs
git add --all
git commit -qm 'Text owner change'
FAIL_TARGET=query-test run_gate text
gate="$(cat target/text/group-validation)"
jq -e '.scope == "group" and any(.stages[]; .target == "cli-test" and .required)' "$gate/plan.json" > /dev/null
for input in Cargo.lock crates/ic-query/src/schema.rs unknown-source; do
  git reset --hard -q "$baseline"
  printf 'shared or unknown change\n' > "$input"
  git add --all
  git commit -qm 'Shared contract or unknown source change'
  run_gate "$(basename "$input")"
  gate="$(cat "target/$(basename "$input")/group-validation")"
  jq -e '.scope == "workspace" and all(.stages[]; .required)' "$gate/plan.json" > /dev/null
done
git reset --hard -q "$baseline"
printf 'CLI change\n' >> crates/ic-query-cli/src/lib.rs
git add --all
git commit -qm 'CLI source checkpoint'
for input in crates/ic-query/src/lib.rs ci/release-validation.json ci/release-validation-scopes.json; do
  DRIFT_INPUT="$input" refuse run_gate input-drift
  [[ ! -e target/input-drift/group-validation ]] || fail 'input drift produced successful evidence'
  git checkout -- "$input"
done
git reset --hard -q "$baseline"
export RELEASE_GROUP=query
printf 'private fixture change\n' >> crates/ic-query-testkit/src/lib.rs
git add --all
git commit -qm 'Private Testkit owner change'
run_gate private-testkit
gate="$(cat target/private-testkit/group-validation)"
jq -e '.scope == "group" and all(.stages[]|select(.target == "query-test" or .target == "cli-test"); .required)' "$gate/plan.json" > /dev/null
git reset --hard -q "$baseline"
printf '# shared change\n' >> Makefile
git add Makefile
git commit -qm 'Shared recipe change'
run_gate shared
gate="$(cat target/shared/group-validation)"
jq -e '.scope == "workspace" and all(.stages[]; .required)' "$gate/plan.json" > /dev/null
jq -r '.stages[]' ci/release-validation.json > "$work_dir/expected"
cmp "$work_dir/expected" target/trace
git reset --hard -q "$baseline"
printf 'other owner\n' >> crates/ic-query-cli/src/lib.rs
git add --all
git commit -qm 'Other owner change'
run_gate other-owner
gate="$(cat target/other-owner/group-validation)"
jq -e '.scope == "workspace"' "$gate/plan.json" > /dev/null
git reset --hard -q "$baseline"
printf '{"hosts":{},"schema":1}\n' > ci/release-validation-hosts.json
git add --all
git commit -qm 'Unqualified host'
run_gate unqualified
gate="$(cat target/unqualified/group-validation)"
jq -e '.scope == "workspace"' "$gate/plan.json" > /dev/null
git reset --hard -q "$baseline"
jq -cS '.stages += ["new-obligation"]' ci/release-validation.json > "$work_dir/new-policy"
cp "$work_dir/new-policy" ci/release-validation.json
git add ci/release-validation.json
git commit -qm 'New full-CI obligation'
run_gate new-stage
gate="$(cat target/new-stage/group-validation)"
jq -e '.scope == "workspace" and any(.stages[]; .target == "new-obligation" and .required)' "$gate/plan.json" > /dev/null
git reset --hard -q "$baseline"
IC_TOOLS=didc run_gate custom-tools
gate="$(cat target/custom-tools/group-validation)"
jq -e '.scope == "workspace" and any(.stages[]; .reason | contains("nondefault tool selection"))' "$gate/plan.json" > /dev/null
RELEASE_GROUP=unknown refuse run_gate unknown
git reset --hard -q "$baseline"
bash scripts/release/qualify-validation.sh > "$work_dir/qualification.log" 2>&1
printf 'Query/CLI product mapping, full expansion, interrupted/input-drift refusal, recovery custody and qualification wiring passed\n'
fixture_complete=true
