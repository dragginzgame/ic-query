#!/usr/bin/env bash
# Shared companions: Makefile scripts/release/adapter.sh scripts/release/publish-workspace.sh scripts/ci/prepare-script-fixture.sh scripts/ci/fixture-lifecycle.sh
set -euo pipefail

# Actual consumer metadata, Git, setup and publication receipts; product gates
# and registry effects are substitutes. Archives use real offline Cargo.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES SHARED_TOOLING_ROOT
unset RELEASE_GROUP RELEASE_GROUPS RELEASE_PUBLISH_TARGET RELEASE_BATCH_ACTIVE
unset RELEASE_SETUP_TARGET RELEASE_SETUP_MODE
unset RELEASE_SOURCE RELEASE_COMMIT RELEASE_KIND RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES
unset CARGO_TARGET_DIR CARGO_HOME CARGO_RELEASE_METADATA_EDITOR FORMAT_CARGO
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
root="$(CDPATH='' cd -P -- "${BASH_SOURCE[0]%/*}/../.." && pwd -P)"
export FROZEN_REAL_CARGO FROZEN_RECORD FROZEN_NORMAL
FROZEN_REAL_CARGO="$(command -v cargo)"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
mkdir -p "$root/target/tmp"
FROZEN_RECORD="$(mktemp -d "$root/target/tmp/ic-query-frozen.XXXXXX")"
# shellcheck disable=SC2329 # Registered as the EXIT trap below.
finish() {
  local status=$? log
  fixture_completion_status "$status" "$fixture_complete" || status=$?
  if [[ "$status" != 0 ]]; then
    for log in interrupted resumed uncertain reconciled completed remote-conflict; do
      [[ ! -f "$FROZEN_RECORD/$log.log" ]] || tail -n 15 "$FROZEN_RECORD/$log.log" >&2 || :
    done
  fi
  printf 'Frozen Query hook evidence retained: %s\n' "$FROZEN_RECORD" >&2 || :
  exit "$status"
}
trap finish EXIT
FROZEN_NORMAL="$FROZEN_RECORD/normal"
mkdir "$FROZEN_NORMAL" "$FROZEN_RECORD/bin" "$FROZEN_RECORD/registry"
git -C "$root" archive HEAD | tar -xf - -C "$FROZEN_NORMAL"
git -C "$root" diff HEAD --name-only -z > "$FROZEN_RECORD/changed-paths"
while IFS= read -r -d '' path; do
  [[ "$path" != Cargo.lock ]] || continue
  [[ -f "$root/$path" && ! -L "$root/$path" ]]
  mkdir -p "$FROZEN_NORMAL/$(dirname "$path")"
  cp -p "$root/$path" "$FROZEN_NORMAL/$path"
done < "$FROZEN_RECORD/changed-paths"
bash "$root/scripts/ci/prepare-script-fixture.sh" "$root" "$FROZEN_NORMAL" \
  Makefile scripts/release/adapter.sh scripts/release/publish-workspace.sh
cp "$root/.shared-tooling.snapshot" "$FROZEN_NORMAL/.shared-tooling.snapshot"
cp "$root/Cargo.lock" "$FROZEN_RECORD/deferred.lock"
cat > "$FROZEN_NORMAL/scripts/release/validation.sh" <<'GATE'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
  check) [[ -f "$2/passed" ]] ;;
  run)
    platform=$("$YQ" -p toml -o json -r '.package[] | select(.name == "ic-platform") | .version' Cargo.lock)
    [[ "$platform" == "$FROZEN_PLATFORM" ]]
    printf '%s %s %s\n' "$RELEASE_GROUP" "$PWD" "$platform" >> "$FROZEN_RECORD/gates"
    if [[ -f "$FROZEN_RECORD/stop-gate" ]]; then
      cp "$FROZEN_RECORD/deferred.lock" "$FROZEN_NORMAL/Cargo.lock"
      perl -0777 -i -pe 's/(\[\[package\]\]\nname = "ic-platform"\nversion = ")[^"]+/${1}999.0.0/ or die "missing Platform record\n"' "$FROZEN_NORMAL/Cargo.lock"
      printf '\n# Deferred while frozen release runs\n' >> "$FROZEN_NORMAL/Cargo.lock"
      rm "$FROZEN_RECORD/stop-gate"
      exit 143
    fi ;;
  *) exit 2 ;;
esac
GATE
cat > "$FROZEN_RECORD/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  clippy) printf '%s %s\n' "$PWD" "$*" >> "$FROZEN_RECORD/lint-dispatch" ;;
  info) [[ -f "$FROZEN_RECORD/registry/${2}" ]] ;;
  publish)
    [[ " $* " == *' --locked '* && " $* " == *' --registry crates-io '* ]]
    [[ "$RELEASE_REMOTE" == origin && "$RELEASE_BRANCH" == main && "$RELEASE_KIND" == patch ]]
    [[ "$RELEASE_COMMIT" == "$(git rev-parse HEAD)" ]]
    while [[ "$1" != --package ]]; do shift; done
    selected="$2" package="${2%@*}"
    [[ "${selected##*@}" == "$RELEASE_VERSION" ]]
    arguments=(package --locked --offline --no-verify -p "$package")
    if [[ "$package" != ic-query ]]; then
      arguments+=(--config 'patch.crates-io.ic-query.path="crates/ic-query"')
    fi
    if [[ "$package" == ic-query-cli ]]; then
      arguments+=(--config 'patch.crates-io.ic-query-text.path="crates/ic-query-text"')
    fi
    "$FROZEN_REAL_CARGO" "${arguments[@]}"
    : > "$FROZEN_RECORD/registry/$selected"
    printf '%s\n' "$selected" >> "$FROZEN_RECORD/publications"
    if [[ -f "$FROZEN_RECORD/stop-publication" ]]; then
      rm "$FROZEN_RECORD/stop-publication"
      exit 143
    fi ;;
  *) exec "$FROZEN_REAL_CARGO" "$@" ;;
esac
CARGO
cat > "$FROZEN_RECORD/bin/curl" <<'CURL'
#!/usr/bin/env bash
set -euo pipefail
url="${!#}"
case "$url" in
  https://crates.io/api/v1/crates/*)
    version="${url##*/}" package="${url%/*}" package="${package##*/}"
    if [[ -f "$FROZEN_RECORD/registry/$package@$version" ]]; then printf 200; else printf 404; fi ;;
  *) echo 'fixture requires already prepared tool archives; network is disabled' >&2; exit 64 ;;
esac
CURL
chmod +x "$FROZEN_RECORD/bin/cargo" "$FROZEN_RECORD/bin/curl"
export PATH="$FROZEN_RECORD/bin:$PATH"
export CARGO_PUBLISH_INDEX_ATTEMPTS=1 CARGO_PUBLISH_INDEX_DELAY_SECONDS=0
export FROZEN_PLATFORM
FROZEN_PLATFORM=$("$root/.tools/host/bin/yq" -p toml -o json -r '.package[] | select(.name == "ic-platform") | .version' "$FROZEN_NORMAL/Cargo.lock")
for group in query cli; do
  version=$(make -s --no-print-directory -C "$FROZEN_NORMAL" release-version "RELEASE_GROUP=$group")
  next=$(perl "$root/scripts/ci/validate-release-inputs.pl" next-version "$version" patch)
  printf '%s\n' "$next" > "$FROZEN_RECORD/$group-version"
  printf '# Frozen fixture\n\n## [%s]\n\n- Qualify consumer hooks.\n' "$next" > "$FROZEN_NORMAL/releases/$group/CHANGELOG.md"
  printf '# Frozen fixture\n\n## %s\n\n- Qualify consumer hooks.\n' "$next" > "$FROZEN_NORMAL/releases/$group/${next%.*}.md"
done
[[ ! -e "$FROZEN_NORMAL/.tools" ]]
git init -q -b main "$FROZEN_NORMAL"
git -C "$FROZEN_NORMAL" config user.name 'Query frozen qualification'
git -C "$FROZEN_NORMAL" config user.email 'qualification@example.invalid'
git -C "$FROZEN_NORMAL" add -- .
git -C "$FROZEN_NORMAL" commit -qm 'Reproducible Query frozen-hook fixture'
git init -q --bare "$FROZEN_RECORD/remote.git"
git -C "$FROZEN_NORMAL" remote add origin "$FROZEN_RECORD/remote.git"
git -C "$FROZEN_NORMAL" push -q origin main
source=$(git -C "$FROZEN_NORMAL" rev-parse HEAD)
for selection in cli,query query,query unknown ''; do
  if make -C "$FROZEN_NORMAL" release-patch "RELEASE_GROUPS=$selection" > "$FROZEN_RECORD/selection-${selection:-empty}.log" 2>&1; then exit 1; fi
done
for retired in RELEASE_FROZEN_GROUPS RELEASE_FROZEN_PUBLISH_TARGET RELEASE_FROZEN_SETUP_TARGET RELEASE_FROZEN_SETUP_MODE FROZEN_RELEASE_ACTIVE; do
  if make -C "$FROZEN_NORMAL" release-patch RELEASE_GROUP=query "$retired=query,cli" > "$FROZEN_RECORD/retired-$retired.log" 2>&1; then exit 1; fi
  grep -q 'Retired release settings' "$FROZEN_RECORD/retired-$retired.log"
done
if make -C "$FROZEN_NORMAL" release-patch RELEASE_GROUP=query RELEASE_GROUPS= > "$FROZEN_RECORD/cleared-roster.log" 2>&1; then exit 1; fi
[[ ! -e "$FROZEN_NORMAL/.git/release-state/frozen" ]]
if make -C "$FROZEN_NORMAL" release-patch release-batch-resume RELEASE_GROUP=query > "$FROZEN_RECORD/conflicting-targets.log" 2>&1; then exit 1; fi
[[ ! -e "$FROZEN_NORMAL/.git/release-state/frozen" ]]
: > "$FROZEN_RECORD/stop-gate"
: > "$FROZEN_RECORD/stop-publication"
if make -C "$FROZEN_NORMAL" release-patch RELEASE_GROUPS=query,cli > "$FROZEN_RECORD/interrupted.log" 2>&1; then exit 1; fi
intents=("$FROZEN_NORMAL"/.git/release-state/frozen/batch-*/intent.json)
[[ ${#intents[@]} == 1 && -f "${intents[0]}" ]]
jq -e '.schema == 2 and .phase == "releasing" and .index == 0 and .setup == "release-candidate-setup"' "${intents[0]}" > /dev/null
[[ ! -e "$FROZEN_RECORD/stop-gate" ]]
batch="${intents[0]%/*}" batch="${batch##*/}"
cp "$FROZEN_NORMAL/Cargo.lock" "$FROZEN_RECORD/expected-deferred.lock"
cp "${intents[0]}" "$FROZEN_RECORD/expected-intent.json"
make -C "$FROZEN_NORMAL" release-batch-status "BATCH=$batch" > "$FROZEN_RECORD/status.log" 2>&1
cmp "$FROZEN_RECORD/expected-intent.json" "${intents[0]}"
jq '.schema = 1' "$FROZEN_RECORD/expected-intent.json" > "${intents[0]}"
if make -C "$FROZEN_NORMAL" release-batch-status "BATCH=$batch" > "$FROZEN_RECORD/old-journal.log" 2>&1; then exit 1; fi
grep -q 'pre-0.31 batches require their original tooling' "$FROZEN_RECORD/old-journal.log"
cp "$FROZEN_RECORD/expected-intent.json" "${intents[0]}"
if make -C "$FROZEN_NORMAL" release-batch-resume "BATCH=$batch" > "$FROZEN_RECORD/resumed.log" 2>&1; then exit 1; fi
jq -e '.phase == "publishing" and .index == 0' "${intents[0]}" > /dev/null
cp "${intents[0]}" "$FROZEN_RECORD/publication-intent.json"
jq 'del(.release_commit)' "$FROZEN_RECORD/publication-intent.json" > "${intents[0]}"
if make -C "$FROZEN_NORMAL" release-batch-resume "BATCH=$batch" > "$FROZEN_RECORD/missing-publication-commit.log" 2>&1; then exit 1; fi
grep -q 'missing recorded publication identity' "$FROZEN_RECORD/missing-publication-commit.log"
cp "$FROZEN_RECORD/publication-intent.json" "${intents[0]}"
grep -q 'install-host-tools.sh.*--check' "$FROZEN_RECORD/resumed.log"
query_version=$(cat "$FROZEN_RECORD/query-version")
[[ -f "$FROZEN_RECORD/registry/ic-query@$query_version" ]]
if make -C "$FROZEN_NORMAL" release-batch-resume "BATCH=$batch" > "$FROZEN_RECORD/uncertain.log" 2>&1; then exit 1; fi
[[ $(wc -l < "$FROZEN_RECORD/publications") == 1 ]]
make -C "$FROZEN_NORMAL" release-batch-resume "BATCH=$batch" PUBLICATION_RESULT=published > "$FROZEN_RECORD/reconciled.log" 2>&1
jq -e '.phase == "complete" and .index == 2' "${intents[0]}" > /dev/null
cmp "$FROZEN_RECORD/expected-deferred.lock" "$FROZEN_NORMAL/Cargo.lock"
[[ $(git -C "$FROZEN_NORMAL" rev-parse HEAD) == "$source" ]]
work="$FROZEN_NORMAL/target/release-frozen/$batch"
gitdir="$FROZEN_NORMAL/.git/release-state/frozen/$batch/repository.git"
[[ $(git --git-dir "$FROZEN_RECORD/remote.git" rev-parse main) == "$(git --git-dir "$gitdir" rev-parse HEAD)" ]]
cli_version=$(cat "$FROZEN_RECORD/cli-version")
printf 'ic-query@%s\nic-query-text@%s\nic-query-cli@%s\n' "$query_version" "$cli_version" "$cli_version" > "$FROZEN_RECORD/expected-publications"
cmp "$FROZEN_RECORD/expected-publications" "$FROZEN_RECORD/publications"
for package in ic-query ic-query-text ic-query-cli; do
  group=cli version="$cli_version"
  [[ "$package" != ic-query ]] || { group=query; version="$query_version"; }
  sha=$(git --git-dir "$gitdir" rev-parse "$group-v$version^{}")
  tar -xOf "$work/target/package/$package-$version.crate" "$package-$version/.cargo_vcs_info.json" |
    jq -e --arg sha "$sha" '.git.sha1 == $sha and (.git.dirty // false) == false' > /dev/null
  receipts=("$work/target/publication/$package."*)
  [[ ${#receipts[@]} == 1 && -f "${receipts[0]}/intent" ]]
  sed -n '2p' "${receipts[0]}/intent" | grep -Fx "$sha" > /dev/null
done
grep -q '^cli ' "$FROZEN_RECORD/gates"
make -C "$FROZEN_NORMAL" release-batch-resume "BATCH=$batch" > "$FROZEN_RECORD/completed.log" 2>&1
cmp "$FROZEN_RECORD/expected-publications" "$FROZEN_RECORD/publications"
if make -C "$FROZEN_NORMAL" release-batch-reconcile "BATCH=$batch" > "$FROZEN_RECORD/dirty-reconcile.log" 2>&1; then exit 1; fi
cmp "$FROZEN_RECORD/expected-deferred.lock" "$FROZEN_NORMAL/Cargo.lock"
git -C "$FROZEN_NORMAL" restore --source=HEAD -- Cargo.lock
before=$(git --git-dir "$FROZEN_RECORD/remote.git" rev-parse main)
if make -C "$FROZEN_NORMAL" release-patch RELEASE_GROUP=query > "$FROZEN_RECORD/remote-conflict.log" 2>&1; then exit 1; fi
[[ $(git --git-dir "$FROZEN_RECORD/remote.git" rev-parse main) == "$before" ]]
cmp "$FROZEN_RECORD/expected-publications" "$FROZEN_RECORD/publications"
make -C "$FROZEN_NORMAL" release-batch-reconcile "BATCH=$batch" > "$FROZEN_RECORD/fast-forward.log" 2>&1
[[ $(git -C "$FROZEN_NORMAL" rev-parse HEAD) == "$before" ]]
[[ $(git -C "$FROZEN_NORMAL" rev-parse "cli-v$cli_version^{}") == "$before" ]]
mkdir "$FROZEN_RECORD/delivered-evidence"
cp -R "$work/target/publication" "$FROZEN_RECORD/delivered-evidence/"
cp "${intents[0]}" "$FROZEN_RECORD/delivered-evidence/intent.json"
for package in ic-query ic-query-text ic-query-cli; do
  version="$cli_version"
  [[ "$package" != ic-query ]] || version="$query_version"
  cp "$work/target/package/$package-$version.crate" "$FROZEN_RECORD/delivered-evidence/"
done
make -C "$FROZEN_NORMAL" release-batch-retire "BATCH=$batch" > "$FROZEN_RECORD/retired.log" 2>&1
[[ ! -e "$work" && ! -e "${intents[0]%/*}" ]]
[[ $(git -C "$FROZEN_NORMAL" rev-parse "cli-v$cli_version^{}") == "$before" ]]
retained=("$FROZEN_NORMAL"/.git/release-state/frozen/batch-*/intent.json)
[[ ${#retained[@]} == 1 && -f "${retained[0]}" ]]
fixture_complete=true
echo 'Frozen Query hooks passed: candidate setup/admission, supplier order, deferred lock, saved publication context, shared status/resume/reconcile/retire, three archive/receipt identities, completed resume and remote conflict'
