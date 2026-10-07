#!/usr/bin/env bash
set -euo pipefail

# Release effects are file-backed stubs, never real Git mutations.
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
bash "$repo_root/scripts/ci/check-release-commands.sh" "$repo_root" make/tools.mk
bash "$repo_root/scripts/ci/test-release-runner.sh"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-release-guards.XXXXXX")"
trap 'if [[ $? == 0 ]]; then rm -rf -- "$work_dir"; else echo "Release guard fixtures retained: $work_dir" >&2; fi' EXIT
export REAL_MAKE REAL_GIT
export REAL_CARGO YQ
REAL_MAKE="$(command -v make)"
REAL_GIT="$(command -v git)"
REAL_CARGO="$(command -v cargo)"
YQ="${YQ:-$repo_root/.tools/host/bin/yq}"
fixture_native_path="$PATH"
mkdir -p "$work_dir/bin"
cat > "$work_dir/bin/git" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
source_sha=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
release_sha=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
tag_sha=cccccccccccccccccccccccccccccccccccccccc
resolve() { if [[ "$1" == HEAD ]]; then cat head; else printf '%s\n' "$1"; fi; }
ancestor() {
  local cursor
  cursor="$(resolve "$2")"
  while [[ "$cursor" != "$1" ]]; do
    [[ -f "commits/$cursor/parent" ]] || return 1
    cursor="$(cat "commits/$cursor/parent")"
  done
}
snapshot() {
  mkdir -p "commits/$1/files"
  cp Cargo.toml Cargo.lock README.md CHANGELOG.md "commits/$1/files/"
  cp -R docs "commits/$1/files/"
}
case "$1" in
  check-ref-format) [[ "$2" == refs/heads/main ]] ;;
  symbolic-ref) echo main ;;
  remote)
    if [[ -f destination ]]; then cat destination
    else echo "${FIXTURE_DESTINATION:-https://example.invalid/ic-query}"; fi ;;
  hash-object) exec "$REAL_GIT" hash-object --stdin ;;
  rev-parse)
    case "${!#}" in
      --show-toplevel) pwd ;;
      release-state) echo .release-state ;;
      HEAD) cat head ;;
      *'^{tree}') name="${!#}"; name="$(resolve "${name%\^\{tree\}}")"; cat "commits/$name/tree" ;;
      refs/tags/*'^{commit}') name="${!#}"; cat "tags/${name#refs/tags/}" ;;
      refs/tags/*) name="${!#}"; [[ -f "tags/${name#refs/tags/}^{commit}" ]]; echo "$tag_sha" ;;
      *) exit 2 ;;
    esac ;;
  diff-index) [[ "${FIXTURE_DIRTY:-}" != yes ]] ;;
  diff)
    case "$2" in
      --binary) cat Cargo.toml Cargo.lock CHANGELOG.md ;;
      --cached) [[ -z "${FIXTURE_CHANGED_PATH:-}" ]] || printf '%s\0' "$FIXTURE_CHANGED_PATH" ;;
      --name-only)
        [[ -z "${FIXTURE_CHANGED_PATH:-}" ]] || printf '%s\0' "$FIXTURE_CHANGED_PATH"
        if [[ "$#" == 5 && "$4" == "$source_sha" && -f descendant ]]; then
          printf 'scripts/release/callback-fix.txt\0'
        fi ;;
      --quiet) [[ "${FIXTURE_DIRTY:-}" != yes ]] ;;
      *) exit 2 ;;
    esac ;;
  ls-files)
    if [[ "${FIXTURE_INVENTORY_FAIL:-}" == yes && "$(cat head)" != "$source_sha" ]]; then
      [[ -z "${FIXTURE_INVENTORY_OUTPUT:-}" ]] || echo "$FIXTURE_INVENTORY_OUTPUT"
      exit 9
    fi
    [[ -z "${FIXTURE_UNTRACKED:-}" ]] || echo "$FIXTURE_UNTRACKED" ;;
  write-tree) echo dddddddddddddddddddddddddddddddddddddddd ;;
  rev-list)
    range="${!#}"; base="${range%..HEAD}"; cursor="$(cat head)"; history=""
    while [[ "$cursor" != "$base" ]]; do
      [[ -f "commits/$cursor/parent" ]] || exit 1
      history="$cursor${history:+$'\n'$history}"
      cursor="$(cat "commits/$cursor/parent")"
    done
    [[ -z "$history" ]] || printf '%s\n' "$history" ;;
  merge-base) ancestor "$3" "$4" ;;
  log)
    case "$3" in
      --format=%P) cat "commits/$4/parent" ;;
      --format=%s) cat "commits/$4/subject" ;;
      *) exit 2 ;;
    esac ;;
  tag)
    case "$2" in
      --list) if [[ -f "tags/$3^{commit}" ]]; then printf '%s\n' "$3"; fi ;;
      -a)
        [[ "$#" == 6 && -f "commits/$4/parent" && "$5" == -m && "$6" == "Release ${3#v}" ]]
        mkdir -p tags
        printf '%s\n' "$4" > "tags/$3^{commit}"
        printf '%s\n' "${3#v}" > tag
        echo tag >> events ;;
      *) exit 2 ;;
    esac ;;
  cat-file) [[ -f tag ]]; echo tag ;;
  ls-remote)
    [[ "${FIXTURE_REMOTE_FAIL:-}" != yes ]] || exit 47
    [[ "$2" == --refs && "$3" == -- && "$4" == https://example.invalid/ic-query ]]
    for ref in "$@"; do
      case "$ref" in
        refs/heads/main) if [[ -f remote-head ]]; then printf '%s\t%s\n' "$(cat remote-head)" "$ref"; fi ;;
        refs/tags/*) if [[ -f remote-tag && "$(cat remote-tag-name)" == "$ref" ]]; then printf '%s\t%s\n' "$(cat remote-tag)" "$ref"; fi ;;
      esac
    done ;;
  add)
    [[ "$#" == 8 && "$2" == -- && "$3" == Cargo.toml && "$4" == Cargo.lock && "$5" == README.md \
      && "$6" == docs/library-usage.md && "$7" == CHANGELOG.md \
      && "$8" == "docs/changelog/$(sed 's/\.[0-9]*$//' candidate).md" ]]
    mkdir -p index
    cp Cargo.toml Cargo.lock README.md CHANGELOG.md index/
    cp -R docs index/
    echo stage >> events ;;
  commit)
    [[ "$#" == 3 && "$2" == -m && "$3" == "Release $(cat candidate)" ]]
    if [[ -d "commits/$release_sha" ]]; then
      release_sha="$(printf '%s\n' "$3" "$(cat head)" | "$REAL_GIT" hash-object --stdin)"
    fi
    snapshot "$release_sha"
    cp head "commits/$release_sha/parent"
    printf '%s\n' "$3" > "commits/$release_sha/subject"
    echo dddddddddddddddddddddddddddddddddddddddd > "commits/$release_sha/tree"
    echo "$release_sha" > head
    echo commit >> events ;;
  push)
    [[ "$#" == 7 && "$2" == --no-follow-tags && "$3" == --atomic && "$4" == -- \
      && "$5" == https://example.invalid/ic-query && "$6" == *:refs/heads/main \
      && "$7" == "refs/tags/v$(cat tag):refs/tags/v$(cat tag)" ]]
    push_head="$(resolve "${6%:refs/heads/main}")"
    if [[ -f remote-head ]]; then ancestor "$(cat remote-head)" "$push_head"; fi
    printf '%s %s\n' "$push_head" "$(cat tag)" >> pushes
    echo push >> events
    [[ "${FIXTURE_BEFORE_PUSH:-}" != yes ]] || exit 47
    echo "$push_head" > remote-head
    echo "$tag_sha" > remote-tag
    printf 'refs/tags/v%s\n' "$(cat tag)" > remote-tag-name
    [[ "${FIXTURE_LOST_PUSH_REPLY:-}" != yes ]] || exit 47 ;;
  show)
    if [[ "$2" == :* ]]; then cat "index/${2#*:}"; exit; fi
    name="$(resolve "${2%%:*}")"
    printf '%s %s\n' "$name" "${2#*:}" >> reads
    cat "commits/$name/files/${2#*:}" ;;
  *) exit 2 ;;
esac
STUB
cat > "$work_dir/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == locate-project ]]; then exec "$REAL_CARGO" "$@"; fi
printf 'cargo %s\n' "$*" >> events
case "$*" in
  'fetch --locked --offline') [[ "${FIXTURE_MISSING_DEPENDENCY:-}" != yes ]] || exit 43 ;;
  'metadata --locked --offline --no-deps --format-version 1') printf '{}\n' ;;
  generate-lockfile) sed "s/1.0.0/${FIXTURE_NEWER_DEPENDENCY:-2.0.0}/" Cargo.lock > changed.lock; mv changed.lock Cargo.lock ;;
  *) exit 2 ;;
esac
STUB
cat > "$work_dir/bin/make" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${2:-}" == -f && "${3:-}" == - ]]; then exec "$REAL_MAKE" "$@"; fi
for argument in "$@"; do
  [[ "$argument" != "${FIXTURE_FAIL_TARGET:-}" ]] || exit 43
  if [[ "$argument" == ci ]]; then
    echo validate >> events
    printf 'retained build output\n' > target/retained-output
    [[ "${CARGO_NET_OFFLINE:-}" == true && "${CHANGELOG_VERSION:-}" == "$(cat candidate)" ]]
    [[ "${FIXTURE_GATE_FAILURE:-}" != yes ]] || exit 43
    exit
  fi
done
"$REAL_MAKE" "$@"
if [[ "${FIXTURE_LOST_PREPARE_REPLY:-}" == yes && "$*" == *release-prepare-version* ]]; then exit 43; fi
STUB
chmod +x "$work_dir/bin/"*
export PATH="$work_dir/bin:$PATH"
fail() { echo "release fixture failed: $*" >&2; exit 1; }
new_fixture() {
  local name="$1" kind="$2" candidate minor
  mkdir -p "$work_dir/$name/scripts/ci" "$work_dir/$name/scripts/release" "$work_dir/$name/docs/changelog" "$work_dir/$name/target"
  cd "$work_dir/$name"
  cp "$repo_root/Makefile" Makefile
  mkdir -p make
  cp "$repo_root/make/tools.mk" make/
  cp "$repo_root/scripts/ci/"{run-release.sh,run-validation-targets.sh,check-make-execution.sh,next-release-version.sh,finalize-release-changelog.awk,rewrite-local-lock-versions.pl,read-cargo-workspace-version.sh} scripts/ci/
  cp "$repo_root/scripts/release/"{adapter.sh,metadata.pl} scripts/release/
  cp "$repo_root/scripts/ci/check-changelog-version.sh" scripts/ci/
  candidate="$(bash scripts/ci/next-release-version.sh 0.46.5 "$kind")"
  minor="${candidate%.*}"
  printf '%s\n' "$candidate" > candidate
  printf 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n' > 'head'
  cat > Cargo.toml <<'MANIFEST'
[workspace.package]
version = "0.46.5"
[workspace.dependencies]
ic-query = { path = "crates/ic-query", version = "0.46.5", default-features = false }
MANIFEST
  cat > Cargo.lock <<'LOCK'
version = 4

[[package]]
name = "external-fixture"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "retained-checksum"

[[package]]
name = "ic-query"
version = "0.46.5"
dependencies = ["external-fixture"]

[[package]]
name = "ic-query-cli"
version = "0.46.5"
dependencies = ["ic-query"]
LOCK
  printf 'ic-query = { version = "0.46", default-features = false }\n' > README.md
  cp README.md docs/library-usage.md
  printf "# Changelog\n\n## [%s]\n\n- Current batch.\n\n## [0.46.x] - 2026-10-04\n\n- \`0.46.5\` historical release.\n" "$candidate" > CHANGELOG.md
  printf '# %s Changelog\n\n## %s\n\n- Current batch.\n' "$minor" "$candidate" > "docs/changelog/$minor.md"
  cp Cargo.lock original-lock
  mkdir -p commits/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/files
  cp Cargo.toml Cargo.lock README.md CHANGELOG.md commits/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/files/
  cp -R docs commits/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/files/
  printf 'retained artifact\n' > target/original-artifact
}
run_release() {
  if [[ "$1" == resume ]]; then
    "$REAL_MAKE" --no-print-directory release-resume "VERSION=$2" > output 2>&1
  else
    "$REAL_MAKE" --no-print-directory "release-$1" > output 2>&1
  fi
}
expect_failure() { if run_release "$@"; then fail "unexpected successful $*"; fi; }
count_event() { awk -v event="$1" '$0 == event { count++ } END { print count+0 }' events; }
check_complete() {
  local candidate minor expected_lock
  candidate="$(cat candidate)"
  minor="${candidate%.*}"
  [[ "$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)" == "$candidate" && "$(cat tag)" == "$candidate" ]]
  [[ "$(count_event commit)" == 1 && "$(count_event tag)" == 1 && "$(count_event push)" == 1 ]]
  [[ "$(tail -n 1 ".release-state/$candidate.plan")" == complete && ! -e .release-state/lock ]]
  [[ -f target/original-artifact && -f target/retained-output ]]
  grep -Fxq "## [$candidate] - $(date -u +%F)" CHANGELOG.md
  grep -Fxq "## $candidate - $(date -u +%F)" "docs/changelog/$minor.md"
  grep -Fq -- "- \`0.46.5\` historical release." CHANGELOG.md
  expected_lock="$(sed "s/0.46.5/$candidate/g" original-lock)"
  [[ "$(cat Cargo.lock)" == "$expected_lock" ]] || fail 'dependency selection changed'
  bash scripts/ci/check-changelog-version.sh "$candidate"
}
commit_fix() {
  local kind="$1" previous candidate minor fix=eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
  previous="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
  candidate="$(bash scripts/ci/next-release-version.sh "$previous" "$kind")"
  minor="${candidate%.*}"
  printf '%s\n' "$candidate" > candidate
  printf '# Changelog\n\n## [%s]\n\n- Reviewed callback fix.\n\n' "$candidate" > pending-notes
  cat CHANGELOG.md >> pending-notes
  mv pending-notes CHANGELOG.md
  if [[ -f "docs/changelog/$minor.md" ]]; then
    printf '# %s Changelog\n\n## %s\n\n- Reviewed callback fix.\n\n' "$minor" "$candidate" > pending-notes
    cat "docs/changelog/$minor.md" >> pending-notes
    mv pending-notes "docs/changelog/$minor.md"
  else
    printf '# %s Changelog\n\n## %s\n\n- Reviewed callback fix.\n' "$minor" "$candidate" > "docs/changelog/$minor.md"
  fi
  printf 'Reviewed later source change\n' >> README.md
  mkdir -p "commits/$fix/files"
  cp head "commits/$fix/parent"
  printf 'Fix release callback\n' > "commits/$fix/subject"
  printf '%s\n' "$fix" > "commits/$fix/tree"
  cp Cargo.toml Cargo.lock README.md CHANGELOG.md "commits/$fix/files/"
  cp -R docs "commits/$fix/files/"
  printf '%s\n' "$fix" > 'head'
  : > descendant
}
save_old_evidence() {
  head -n 10 .release-state/0.47.0.plan > original-plan
  cp .release-state/0.47.0.validation original-binding
  cp -R .release-state/0.47.0.verify.* original-evidence
  if [[ -f 'tags/v0.47.0^{commit}' ]]; then cp 'tags/v0.47.0^{commit}' original-tag; fi
}
check_old_evidence() {
  head -n 10 .release-state/0.47.0.plan > reconciled-plan
  cmp original-plan reconciled-plan
  cmp original-binding .release-state/0.47.0.validation
  if [[ -f original-tag ]]; then cmp original-tag 'tags/v0.47.0^{commit}'; fi
  [[ "$(cat 'tags/v0.47.0^{commit}')" == bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb ]]
  diff -r original-evidence .release-state/0.47.0.verify.*
  [[ "$(tail -n 1 .release-state/0.47.0.plan)" == complete ]]
  grep -Fxq 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb Cargo.lock' reads
}
for kind in patch minor major; do
  new_fixture "success-$kind" "$kind"
  FIXTURE_NEWER_DEPENDENCY=9.0.0 FIXTURE_CHANGED_PATH=README.md run_release "$kind" \
    || { cat output; fail "$kind"; }
  check_complete
  awk '/^(validate|stage|commit|tag|push)$/ { print }' events > observed
  printf '%s\n' validate stage commit tag push > expected
  cmp expected observed
done
new_fixture prepared-payload-retry patch
FIXTURE_LOST_PREPARE_REPLY=yes expect_failure patch
cp Cargo.lock saved-lock
run_release patch || { cat output; fail 'prepared Query payload retry'; }
check_complete
cmp saved-lock Cargo.lock
[[ "$(count_event validate)" == 1 ]] || fail 'prepared Query payload required new validation'
for next_kind in patch minor major resume; do
  for outcome in before-push lost-reply; do
    new_fixture "descendant-$next_kind-$outcome" minor
    case "$outcome" in
      before-push) FIXTURE_BEFORE_PUSH=yes expect_failure minor ;;
      lost-reply) FIXTURE_LOST_PUSH_REPLY=yes expect_failure minor ;;
    esac
    save_old_evidence
    if [[ "$next_kind" == resume ]]; then commit_fix patch; else commit_fix "$next_kind"; fi
    cp Cargo.lock original-lock
    if [[ "$next_kind" == resume ]]; then
      run_release resume 0.47.0 || { cat output; fail 'descendant exact resume'; }
      [[ "$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)" == 0.47.0 && "$(count_event validate)" == 1 ]]
      [[ "$(count_event commit)" == 1 && "$(count_event tag)" == 1 ]]
      [[ ! -e .release-state/0.47.1.plan ]]
    else
      run_release "$next_kind" || { cat output; fail 'descendant ordinary increment'; }
      [[ "$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)" == "$(cat candidate)" ]]
      [[ "$(count_event validate)" == 2 && "$(count_event commit)" == 2 && "$(count_event tag)" == 2 ]]
      [[ "$(tail -n 1 ".release-state/$(cat candidate).plan")" == complete ]]
      sed "s/0.47.0/$(cat candidate)/g" original-lock > expected-lock
      cmp expected-lock Cargo.lock
      next_evidence="$(cat ".release-state/$(cat candidate).validation")"
      [[ "$(sed -n '6p' "$next_evidence/identity")" == eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee ]]
      bash scripts/ci/check-changelog-version.sh "$(cat candidate)"
    fi
    check_old_evidence
    if [[ "$outcome" == before-push ]]; then
      [[ "$(sed -n '2p' pushes)" == 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb 0.47.0' ]]
    elif [[ "$next_kind" == resume ]]; then
      [[ "$(count_event push)" == 1 ]]
    else
      [[ "$(count_event push)" == 2 ]]
    fi
    [[ -f target/original-artifact && -f target/retained-output && ! -e .release-state/lock ]]
  done
done
for target in release-committed-check release-tagged-check; do
  new_fixture "descendant-late-$target" minor
  FIXTURE_FAIL_TARGET="$target" expect_failure minor
  [[ "$(count_event commit)" == 1 && "$(count_event push)" == 0 ]]
  save_old_evidence
  commit_fix patch
  run_release patch || { cat output; fail 'historical late callback retry'; }
  check_old_evidence
  [[ "$(count_event validate)" == 2 && "$(count_event commit)" == 2 && "$(count_event tag)" == 2 ]]
  [[ "$(cat tag)" == 0.47.1 && "$(tail -n 1 .release-state/0.47.1.plan)" == complete ]]
done
new_fixture descendant-gate-retry minor
FIXTURE_BEFORE_PUSH=yes expect_failure minor
save_old_evidence
commit_fix patch
FIXTURE_GATE_FAILURE=yes expect_failure patch
check_old_evidence
[[ "$(count_event commit)" == 1 && ! -e .release-state/0.47.1.plan ]]
failed_next_evidence=(.release-state/0.47.1.verify.*)
cp "${failed_next_evidence[0]}/validation.log" failed-next-log
run_release patch || { cat output; fail 'descendant new gate retry'; }
cmp failed-next-log "${failed_next_evidence[0]}/validation.log"
[[ "$(count_event validate)" == 3 && "$(count_event commit)" == 2 && "$(count_event tag)" == 2 ]]
check_old_evidence
new_fixture descendant-remote-tip minor
FIXTURE_BEFORE_PUSH=yes expect_failure minor
save_old_evidence
commit_fix patch
cp head remote-head
run_release resume 0.47.0 || { cat output; fail 'confirmed remote descendant'; }
[[ "$(cat remote-head)" == eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee ]]
[[ "$(sed -n '2p' pushes)" == 'eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee 0.47.0' ]]
check_old_evidence
for output in empty partial; do
  new_fixture "inventory-$output" patch
  if [[ "$output" == partial ]]; then
    FIXTURE_INVENTORY_FAIL=yes FIXTURE_INVENTORY_OUTPUT=caller-owned.txt expect_failure patch
  else
    FIXTURE_INVENTORY_FAIL=yes expect_failure patch
  fi
  [[ "$(count_event commit)" == 1 && ! -e tag && ! -e remote-head ]]
  [[ "$(tail -n 1 .release-state/0.46.6.plan)" == commit ]]
  run_release patch || { cat output; fail 'inventory recovery'; }
  check_complete
done
for conflict in payload history tag destination remote-unavailable remote-diverged inventory; do
  new_fixture "descendant-conflict-$conflict" minor
  FIXTURE_BEFORE_PUSH=yes expect_failure minor
  commit_fix patch
  cp .release-state/0.47.0.plan saved-plan
  case "$conflict" in
    payload) printf 'conflicting release payload\n' >> commits/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb/files/README.md ;;
    history) printf 'Unexpected subject\n' > commits/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb/subject ;;
    tag) printf 'eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee\n' > 'tags/v0.47.0^{commit}' ;;
    destination) export FIXTURE_DESTINATION=https://example.invalid/other ;;
    remote-unavailable) export FIXTURE_REMOTE_FAIL=yes ;;
    remote-diverged) printf 'ffffffffffffffffffffffffffffffffffffffff\n' > remote-head ;;
    inventory) export FIXTURE_INVENTORY_FAIL=yes ;;
  esac
  expect_failure patch
  cmp saved-plan .release-state/0.47.0.plan
  [[ "$(count_event commit)" == 1 && "$(count_event tag)" == 1 && "$(count_event push)" == 1 ]]
  [[ "$(count_event validate)" == 1 && ! -e .release-state/0.47.1.plan ]]
  unset FIXTURE_DESTINATION FIXTURE_REMOTE_FAIL FIXTURE_INVENTORY_FAIL
done
for notes in root detail; do
  new_fixture "candidate-mismatch-$notes" patch
  if [[ "$notes" == root ]]; then notes_path=CHANGELOG.md; else notes_path=docs/changelog/0.46.md; fi
  printf '# Changelog\n\n## [9.9.9]\n' > "$notes_path"
  cp "$notes_path" "commits/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/files/$notes_path"
  cp "$notes_path" conflicting-notes
  expect_failure patch
  cmp conflicting-notes "$notes_path"
  cmp original-lock Cargo.lock
  [[ ! -e events && ! -e .release-state/0.46.6.plan && ! -e .release-state/0.46.6.validation ]]
done
for failure in gate missing-dependency dirty untracked; do
  new_fixture "failure-$failure" patch
  case "$failure" in
    gate) FIXTURE_GATE_FAILURE=yes expect_failure patch ;;
    missing-dependency) FIXTURE_MISSING_DEPENDENCY=yes expect_failure patch ;;
    dirty) FIXTURE_DIRTY=yes expect_failure patch ;;
    untracked) FIXTURE_UNTRACKED=caller-owned.txt expect_failure patch ;;
  esac
  [[ "$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)" == 0.46.5 && ! -e tag && ! -e remote-head ]]
  cmp original-lock Cargo.lock
  [[ -f target/original-artifact && ! -e .release-state/0.46.6.plan && ! -e .release-state/lock ]]
  if [[ "$failure" == gate ]]; then
    failed_logs=(.release-state/0.46.6.verify.*/validation.log)
    [[ -f "${failed_logs[0]}" ]] || fail 'failed gate log was discarded'
    run_release patch || { cat output; fail 'fresh gate retry'; }
    [[ -f "${failed_logs[0]}" && "$(count_event validate)" == 2 ]]
    check_complete
  fi
done
for conflict in payload dependency identity retained-inputs unrelated; do
  new_fixture "conflict-$conflict" patch
  FIXTURE_LOST_PREPARE_REPLY=yes expect_failure patch
  case "$conflict" in
    payload) echo 'unvalidated edit' >> README.md ;;
    dependency) sed 's/1.0.0/2.0.0/' Cargo.lock > changed; mv changed Cargo.lock ;;
    identity) echo invalid > .release-state/0.46.6.verify.*/identity ;;
    retained-inputs) echo 'changed evidence' >> .release-state/0.46.6.verify.*/before/Cargo.lock ;;
    unrelated) export FIXTURE_CHANGED_PATH=src/unrelated.rs ;;
  esac
  expect_failure patch
  [[ ! -e tag && ! -e remote-head && -f target/retained-output ]]
  unset FIXTURE_CHANGED_PATH
done
echo 'IC Query release adapter fixtures passed'

# A distinct parent checkout catches leaked logger identity without running CI.
metadata_context="$work_dir/metadata-context"
mkdir -p "$metadata_context"
cat > "$metadata_context/Makefile" <<'MAKE'
.PHONY: metadata-check ci
metadata-check:
	@bash "$(FIXTURE_REPOSITORY_ROOT)/scripts/ci/check-release-metadata.sh"
ci:
	@echo 'error: metadata fixture reached its parent gate' >&2
	@echo parent-gate-reached > parent-routing.log
	@exit 7
MAKE
PATH="$fixture_native_path" FIXTURE_REPOSITORY_ROOT="$repo_root" \
  VALIDATION_REPOSITORY_ROOT="$metadata_context" \
  VALIDATION_FAILURE_LOG_DIR="$metadata_context/failures" \
  bash "$repo_root/scripts/ci/run-validation-targets.sh" --fail-fast metadata-check
[[ ! -e "$metadata_context/parent-routing.log" ]] \
  || fail 'metadata fixture validation escaped its checkout'
