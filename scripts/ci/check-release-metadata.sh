#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-release-metadata.XXXXXX")"
trap 'rm -rf "$work_dir"' EXIT
fail() { echo "release metadata fixture failed: $*" >&2; exit 1; }

# Reuse history and a real index. A synthetic source tree models validated
# inputs without making a commit; only Cargo and the expensive gate are stubbed.
git clone --quiet --shared "$repo_root" "$work_dir/repository"
cd "$work_dir/repository"
mkdir -p scripts/release scripts/ci "$work_dir/bin"
cp "$repo_root/Makefile" Makefile
cp "$repo_root/scripts/release/"{adapter.sh,metadata.pl} scripts/release/
cp "$repo_root/scripts/ci/"{next-release-version.sh,finalize-release-changelog.awk,run-validation-targets.sh} scripts/ci/
cat > "$work_dir/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == 'metadata --locked --offline --no-deps --format-version 1' ]]
printf '{}\n'
STUB
chmod +x "$work_dir/bin/cargo"
export PATH="$work_dir/bin:$PATH"
export RELEASE_KIND=minor RELEASE_REMOTE=origin RELEASE_BRANCH=main RELEASE_DATE=2026-10-06
export RELEASE_PREVIOUS RELEASE_VERSION RELEASE_SOURCE
RELEASE_PREVIOUS="$(perl scripts/release/metadata.pl version)"
RELEASE_VERSION="$(bash scripts/ci/next-release-version.sh "$RELEASE_PREVIOUS" "$RELEASE_KIND")"
detail="docs/changelog/${RELEASE_VERSION%.*}.md"
printf '# Changelog\n\n## [%s]\n\n- Reviewed fixture change.\n' "$RELEASE_VERSION" > CHANGELOG.md
printf '# Detailed changelog\n\n## %s\n\n- Reviewed fixture change.\n' "$RELEASE_VERSION" > "$detail"
perl scripts/release/metadata.pl preflight
for notes in CHANGELOG.md "$detail"; do
  cp "$notes" "$work_dir/pending-notes"
  printf '# Changelog\n\n## [9.9.9]\n' > "$notes"
  if perl scripts/release/metadata.pl preflight > "$work_dir/candidate.log" 2>&1; then
    fail 'conflicting candidate passed preflight'
  fi
  cmp "$notes" <(printf '# Changelog\n\n## [9.9.9]\n')
  cp "$work_dir/pending-notes" "$notes"
done
git add -- Makefile scripts/release/adapter.sh scripts/release/metadata.pl \
  scripts/ci/next-release-version.sh scripts/ci/finalize-release-changelog.awk \
  scripts/ci/run-validation-targets.sh \
  Cargo.toml Cargo.lock README.md docs/library-usage.md CHANGELOG.md "$detail"
RELEASE_SOURCE="$(git write-tree)"
evidence=".git/release-state/$RELEASE_VERSION.verify.real-git"
mkdir -p "$evidence"
printf '%s\n' validation-1 "$RELEASE_KIND" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" \
  "$RELEASE_DATE" "$RELEASE_SOURCE" "$RELEASE_REMOTE" "$RELEASE_BRANCH" > "$evidence/identity"
perl scripts/release/metadata.pl capture "$evidence"
: > "$evidence/passed"
printf '%s\n' "$evidence" > ".git/release-state/$RELEASE_VERSION.validation"
perl scripts/release/metadata.pl prepare "$evidence"

# The working payload is prepared but the real index still holds the source.
if bash scripts/release/adapter.sh commit-check > "$work_dir/stale-index.log" 2>&1; then
  fail 'stale staged metadata passed the commit boundary'
fi
git add -- Cargo.toml Cargo.lock README.md docs/library-usage.md CHANGELOG.md "$detail"
bash scripts/release/adapter.sh commit-check

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
	@echo release-gate-failure-marker
	@exit 7
MAKE
for attempt in first second; do
  if bash scripts/release/adapter.sh verify > "$work_dir/$attempt-gate.log" 2>&1; then
    fail 'failed real Make gate passed validation'
  fi
  failed_logs=(.git/release-state/"$RELEASE_VERSION".verify.*/validation-failures/*-0-ci.log)
  if [[ "$attempt" == first ]]; then
    [[ "${#failed_logs[@]}" == 1 ]]
    first_log="${failed_logs[0]}"
    cp "$first_log" "$work_dir/first-retained.log"
  fi
done
[[ "${#failed_logs[@]}" == 2 ]]
grep -Fq release-gate-failure-marker "$first_log"
cmp "$first_log" "$work_dir/first-retained.log"
printf '%s\n' "$evidence" > "$work_dir/expected-binding"
cmp "$work_dir/expected-binding" ".git/release-state/$RELEASE_VERSION.validation"
echo 'IC Query real-Git metadata and actual validation-log fixtures passed'
