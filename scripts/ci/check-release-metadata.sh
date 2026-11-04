#!/usr/bin/env bash
set -euo pipefail

# This fixture owns its Make selections and nested logger's checkout identity.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH

repo_root="$(CDPATH="" cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
export REAL_CARGO YQ
REAL_CARGO="$(command -v cargo)"
YQ="${YQ:-$repo_root/.tools/host/bin/yq}"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-release-metadata.XXXXXX")"
work_dir="$(CDPATH="" cd -P -- "$work_dir" && printf '%s/.' "$PWD")"
work_dir="${work_dir%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "Release metadata fixtures retained"' EXIT
export TMPDIR="${work_dir%/*}"
if [[ -z "${CARGO_RELEASE_METADATA_EDITOR:-}" ]]; then
  CARGO_RELEASE_METADATA_EDITOR="$(bash "$repo_root/scripts/dev/install-rust-tools.sh" --consumer "$repo_root" \
    --package ic-build --lockfile "$repo_root/Cargo.lock" --bin ic-build-release-metadata --profile debug --features cli --check)"
  export CARGO_RELEASE_METADATA_EDITOR
fi
fail() { echo "release metadata fixture failed: $*" >&2; exit 1; }

# Reuse history and a real index. A synthetic source tree models validated
# inputs without making a commit; only Cargo and the expensive gate are stubbed.
git clone --quiet --shared "$repo_root" "$work_dir/repository"
cd "$work_dir/repository"
mkdir -p scripts/release scripts/ci make "$work_dir/bin"
bash "$repo_root/scripts/ci/prepare-script-fixture.sh" "$repo_root" "$PWD" \
  Makefile scripts/release/adapter.sh scripts/ci/cargo-release-group.sh
cp "$repo_root/Cargo.toml" "$repo_root/Cargo.lock" .
cp -R "$repo_root/crates" "$repo_root/releases" .
cp "$repo_root/ci/cargo-release-groups.json" ci/
# Metadata fixtures substitute the successful gate; actual evidence admission
# belongs to check-release-validation.sh.
cat > scripts/release/validation.sh <<'STUB'
set -euo pipefail
case "$1" in
  check) [[ -f "$2/passed" ]] ;;
  run) bash scripts/ci/run-validation-targets.sh --fail-fast ci ;;
  *) exit 2 ;;
esac
STUB
cat > "$work_dir/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$*" == "metadata --locked --offline --format-version 1" ]]; then exec "$REAL_CARGO" "$@"; fi
if [[ "$1" == locate-project ]]; then exec "$REAL_CARGO" "$@"; fi
if [[ "$*" == 'sort --workspace --grouped --check' ]]; then
  printf '%s\n' "$PWD" >> "$ORDER_TRACE"
  cp Cargo.toml "$ORDER_TRACE.manifest"
  [[ "${FAIL_MANIFEST_ORDER:-}" != yes ]] || exit 79
  exit 0
fi
[[ "$*" == 'metadata --locked --offline --no-deps --format-version 1' ]] || exit 1
printf '{}\n'
STUB
chmod +x "$work_dir/bin/cargo"
export PATH="$work_dir/bin:$PATH"
export ORDER_TRACE="$work_dir/manifest-order-trace"
export RELEASE_GROUP="${RELEASE_GROUP:-query}"
export RELEASE_KIND=minor RELEASE_REMOTE=origin RELEASE_BRANCH=main RELEASE_DATE=2026-10-06
export RELEASE_PREVIOUS RELEASE_VERSION RELEASE_SOURCE
RELEASE_PREVIOUS="$(bash scripts/ci/cargo-release-group.sh version "$RELEASE_GROUP")"
RELEASE_VERSION="$(bash scripts/ci/next-release-version.sh "$RELEASE_PREVIOUS" "$RELEASE_KIND")"
detail="releases/$RELEASE_GROUP/${RELEASE_VERSION%.*}.md"
ledger="releases/$RELEASE_GROUP/CHANGELOG.md"
printf '# Changelog\n\n## [%s]\n\n- Reviewed fixture change.\n' "$RELEASE_VERSION" > "$ledger"
printf '# Detailed changelog\n\n## %s\n\n- Reviewed fixture change.\n' "$RELEASE_VERSION" > "$detail"
# The disposable candidate can start a new minor after either group advances.
jq --arg group "$RELEASE_GROUP" --arg detail "$detail" \
  '.groups |= map(if .name == $group then .metadata_files = [$detail] else . end)' \
  ci/cargo-release-groups.json > "$work_dir/group-policy"
mv "$work_dir/group-policy" ci/cargo-release-groups.json
bash scripts/ci/cargo-release-group.sh files "$RELEASE_GROUP" > "$work_dir/release-files"
payloads=()
while IFS= read -r -d '' path; do payloads+=("$path"); done < "$work_dir/release-files"
printf 'Reviewed examples:\nic-query = { version = "%s", default-features = false }\nic-query = ">=0.40, <1"\nHistorical example: 0.40.\n' "${RELEASE_VERSION%.*}" > README.md
printf 'Library usage is maintained through source review.\n' > docs/library-usage.md
cp README.md "$work_dir/reviewed-readme"
cp docs/library-usage.md "$work_dir/reviewed-library-usage"
perl scripts/release/metadata.pl preflight
for notes in "$ledger" "$detail"; do
  cp "$notes" "$work_dir/pending-notes"
  printf '# Changelog\n\n## [9.9.9]\n' > "$notes"
  if perl scripts/release/metadata.pl preflight > "$work_dir/candidate.log" 2>&1; then
    fail 'conflicting candidate passed preflight'
  fi
  cmp "$notes" <(printf '# Changelog\n\n## [9.9.9]\n')
  cp "$work_dir/pending-notes" "$notes"
done
# Capture the complete prepared source closure in this disposable clone only.
git add --all -- .
RELEASE_SOURCE="$(git write-tree)"
evidence=".git/release-state/${RELEASE_GROUP}-$RELEASE_VERSION.verify.real-git"
mkdir -p "$evidence"
printf '%s\n' validation-1 "$RELEASE_KIND" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" \
  "$RELEASE_DATE" "$RELEASE_SOURCE" "$RELEASE_REMOTE" "$RELEASE_BRANCH" "$RELEASE_GROUP" "$(perl -MDigest::SHA=sha256_hex -0777 -ne 'print sha256_hex($_)' ci/cargo-release-groups.json)" > "$evidence/identity"
perl scripts/release/metadata.pl capture "$evidence"
: > "$evidence/passed"
printf '%s\n' "$evidence" > ".git/release-state/${RELEASE_GROUP}-$RELEASE_VERSION.validation"
cp "$evidence/before/Cargo.toml" "$work_dir/reviewed-manifest"
printf '\n# changed retained input\n' >> "$evidence/before/Cargo.toml"
if perl scripts/release/metadata.pl prepare "$evidence" > "$work_dir/changed-input.log" 2>&1; then
  fail 'changed retained source passed rendering'
fi
cmp "$work_dir/reviewed-manifest" Cargo.toml
cp "$work_dir/reviewed-manifest" "$evidence/before/Cargo.toml"
# Refusal must leave all authenticated inputs intact, including before retries.
for refusal in editor allowance bound; do
  status=0
  case "$refusal" in
    editor) CARGO_RELEASE_METADATA_EDITOR=/missing/editor perl scripts/release/metadata.pl prepare "$evidence" > "$work_dir/$refusal.log" 2>&1 || status=$? ;;
    allowance) CARGO_RELEASE_METADATA_MAX_BYTES=0 perl scripts/release/metadata.pl prepare "$evidence" > "$work_dir/$refusal.log" 2>&1 || status=$? ;;
    bound) CARGO_RELEASE_METADATA_MAX_BYTES=1 perl scripts/release/metadata.pl prepare "$evidence" > "$work_dir/$refusal.log" 2>&1 || status=$? ;;
  esac
  [[ "$status" != 0 ]] || fail "invalid editor selection or input bound admitted: $refusal"
  for payload in "${payloads[@]}"; do
    cmp "$evidence/before/$payload" "$payload"
  done
done
unselected_paths=()
while IFS= read -r path; do
  unselected_paths+=("$path")
  mkdir -p "$work_dir/unselected/${path%/*}"
  cp "$path" "$work_dir/unselected/$path"
done < <(jq -r --arg group "$RELEASE_GROUP" '.groups[] | select(.name != $group) | .changelog, (.members[] | "crates/\(.)/Cargo.toml")' ci/cargo-release-groups.json)
perl scripts/release/metadata.pl prepare "$evidence"
for path in "${unselected_paths[@]}"; do cmp "$work_dir/unselected/$path" "$path"; done
if [[ "$RELEASE_GROUP" == query ]]; then selected_packages=(ic-query ic-query-testkit)
else selected_packages=(ic-query-text ic-query-cli); fi
for package in "${selected_packages[@]}"; do
  selected_version="$(awk -F '"' -v package="$package" '
    /^\[\[package\]\]$/ { name = "" }
    /^name = / { name = $2 }
    /^version = / && name == package { print $2 }
  ' Cargo.lock)"
  [[ "$selected_version" == "$RELEASE_VERSION" ]] || fail "workspace lock version differs: $package"
  catalog_version="$(PACKAGE="$package" "$YQ" -er '.workspace.dependencies[strenv(PACKAGE)].version' Cargo.toml)"
  [[ "$catalog_version" == "$RELEASE_VERSION" ]] || fail "workspace catalog version differs: $package"
done
perl scripts/release/metadata.pl check "$evidence"
mkdir -p "$work_dir/prepared"
for payload in "${payloads[@]}"; do
  if [[ "$payload" == */* ]]; then mkdir -p "$work_dir/prepared/${payload%/*}"; fi
  cp "$payload" "$work_dir/prepared/$payload"
done
for interrupted in "${payloads[@]}"; do
  cp "$evidence/before/$interrupted" "$interrupted"
  perl scripts/release/metadata.pl admit "$evidence"
  perl scripts/release/metadata.pl prepare "$evidence"
  for payload in "${payloads[@]}"; do
    cmp "$work_dir/prepared/$payload" "$payload"
  done
done
cmp "$work_dir/reviewed-readme" README.md
cmp "$work_dir/reviewed-library-usage" docs/library-usage.md

# The working payload is prepared but the real index still holds the source.
if bash scripts/release/adapter.sh commit-check > "$work_dir/stale-index.log" 2>&1; then
  fail 'stale staged metadata passed the commit boundary'
fi
git add -- "${payloads[@]}"
bash scripts/release/adapter.sh commit-check

status=0
FAIL_MANIFEST_ORDER=yes bash scripts/release/adapter.sh prepared-check \
  > "$work_dir/order-refusal.log" 2>&1 || status=$?
[[ "$status" == 79 ]] || fail 'prepared check hid manifest-order refusal'

# Committed admission must sort the selected bytes, independently of the worktree.
export RELEASE_COMMIT
RELEASE_COMMIT="$(git write-tree)"
printf '\n# unrelated working manifest evidence\n' >> Cargo.toml
bash scripts/release/adapter.sh committed-check
cmp "$work_dir/prepared/Cargo.toml" "$ORDER_TRACE.manifest"
[[ "$(tail -n 1 "$ORDER_TRACE")" == "$PWD/.git/release-state/${RELEASE_GROUP}-$RELEASE_VERSION.manifest-order."*/source ]] \
  || fail 'committed ordering did not select the retained release source'
cp "$work_dir/prepared/Cargo.toml" Cargo.toml
status=0
FAIL_MANIFEST_ORDER=yes bash scripts/release/adapter.sh committed-check \
  > "$work_dir/committed-order-refusal.log" 2>&1 || status=$?
[[ "$status" == 79 ]] || fail 'committed check hid manifest-order refusal'
unset RELEASE_COMMIT

# A staged replacement with the working file restored hides from HEAD-to-worktree.
original="$(git rev-parse "$RELEASE_SOURCE:AGENTS.md")"
replacement="$(git rev-parse "$RELEASE_SOURCE:README.md")"
git update-index --cacheinfo "100644,$replacement,AGENTS.md"
git diff --quiet "$RELEASE_SOURCE" -- AGENTS.md
if git diff --cached --quiet "$RELEASE_SOURCE" -- AGENTS.md; then
  fail 'hidden staged-change precondition was not established'
fi
for operation in prepared-check commit-check; do
  if bash scripts/release/adapter.sh "$operation" > "$work_dir/$operation.log" 2>&1; then
    fail "hidden staged work passed $operation"
  fi
done
git update-index --cacheinfo "100644,$original,AGENTS.md"
bash scripts/release/adapter.sh commit-check

# Return to the selected inputs for actual adapter/logger failures and retries.
git read-tree "$RELEASE_SOURCE"
git checkout-index --force --all
cat >> Makefile <<'MAKE'
ci:
	@test -z "$${IC_QUERY_RELEASE_PREPARE_CACHE+x}"
	@echo release-gate-failure-marker
	@exit 7
MAKE
for attempt in first second; do
  if IC_QUERY_RELEASE_PREPARE_CACHE=1 bash scripts/release/adapter.sh verify > "$work_dir/$attempt-gate.log" 2>&1; then
    fail 'failed real Make gate passed validation'
  fi
  failed_logs=(.git/release-state/"${RELEASE_GROUP}-$RELEASE_VERSION".verify.*/validation-failures/*-0-ci.log)
  if [[ "$attempt" == first ]]; then
    [[ "${#failed_logs[@]}" == 1 ]] || exit 1
    first_log="${failed_logs[0]}"
    cp "$first_log" "$work_dir/first-retained.log"
  fi
done
[[ "${#failed_logs[@]}" == 2 ]] || exit 1
grep -Fq release-gate-failure-marker "$first_log" \
  || fail 'validation logger did not run the fixture gate'
cmp "$first_log" "$work_dir/first-retained.log"
printf '%s\n' "$evidence" > "$work_dir/expected-binding"
cmp "$work_dir/expected-binding" ".git/release-state/${RELEASE_GROUP}-$RELEASE_VERSION.validation"
echo 'IC Query real-Git metadata and actual validation-log fixtures passed'
fixture_complete=true
