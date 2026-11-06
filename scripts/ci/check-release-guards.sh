#!/usr/bin/env bash
set -euo pipefail

# Release effects are file-backed stubs, never real Git mutations.
export RELEASE_DELIVERY=direct RELEASE_GROUP=query RELEASE_BATCH_ACTIVE=1
repo_root="$(CDPATH="" cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
mkdir -p "${TMPDIR:-$repo_root/target/tmp}"
work_dir="$(mktemp -d "${TMPDIR:-$repo_root/target/tmp}/ic-query-release-guards.XXXXXX")"
work_dir="$(CDPATH="" cd -P -- "$work_dir" && printf '%s/.' "$PWD")"
work_dir="${work_dir%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "Release guard fixtures retained"' EXIT
export TMPDIR="${work_dir%/*}"
if [[ -z "${CARGO_RELEASE_METADATA_EDITOR:-}" ]]; then
  CARGO_RELEASE_METADATA_EDITOR="$(bash "$repo_root/scripts/dev/install-rust-tools.sh" --consumer "$repo_root" \
    --package ic-build --lockfile "$repo_root/Cargo.lock" --bin ic-build-release-metadata --profile debug --features cli --check)"
  export CARGO_RELEASE_METADATA_EDITOR
fi
if [[ "${1:-}" != --assert-completion ]]; then
  # Query owns cache preparation and actual Make routing; generic runner cases stay upstream.
  bash "$repo_root/scripts/ci/check-release-cache.sh"
  bash "$repo_root/scripts/ci/check-release-commands.sh" "$repo_root" make/consumer.mk make/tools.mk make/release.mk make/rust-format.mk make/execution.mk scripts/ci/check-make-execution.sh scripts/ci/run-formatting.sh
fi
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
  cp Cargo.toml Cargo.lock README.md releases/query/CHANGELOG.md "commits/$1/files/"
  cp -R docs ci crates releases "commits/$1/files/"
}
case "$1" in
  for-each-ref) printf '\n' ;;
  check-ref-format) [[ "$2" == refs/heads/main ]] ;;
  symbolic-ref) echo main ;;
  remote)
    if [[ -f destination ]]; then cat destination
    else echo "${FIXTURE_DESTINATION:-https://example.invalid/ic-query}"; fi ;;
  hash-object) exec "$REAL_GIT" hash-object --stdin ;;
  rev-parse)
    case "${!#}" in
      --show-toplevel) pwd ;;
      --show-prefix) printf '\n' ;;
      release-state) echo .release-state ;;
      HEAD)
        cat head
        if [[ -n "${FIXTURE_HEAD_FAILURE_AT:-}" ]]; then
          observation=0
          [[ ! -f head-observations ]] || observation="$(cat head-observations)"
          observation=$((observation + 1))
          printf '%s\n' "$observation" > head-observations
          if [[ "$observation" == "$FIXTURE_HEAD_FAILURE_AT" ]]; then
            echo 'fixture source observation failed' >&2
            exit 23
          fi
        fi ;;
      *'^{tree}') name="${!#}"; name="$(resolve "${name%\^\{tree\}}")"; cat "commits/$name/tree" ;;
      refs/tags/*'^{commit}') name="${!#}"; cat "tags/${name#refs/tags/}" ;;
      refs/tags/*) name="${!#}"; [[ -f "tags/${name#refs/tags/}^{commit}" ]] || exit 1; echo "$tag_sha" ;;
      *'^{commit}') name="${!#}"; name="$(resolve "${name%\^\{commit\}}")";
        [[ -d "commits/$name" ]] || exit 1; printf '%s\n' "$name" ;;
      *) exit 2 ;;
    esac ;;
  status)
    [[ "$*" == 'status --porcelain=v1 -z --untracked-files=all' ]] || exit 1
    if [[ "${FIXTURE_STATUS_FAIL:-}" == yes ]]; then
      echo 'fixture Git status observation failed' >&2
      exit 9
    fi
    [[ "${FIXTURE_DIRTY:-}" != yes ]] || printf ' M Cargo.lock\0'
    [[ "${FIXTURE_STAGED:-}" != yes ]] || printf 'MM README.md\0'
    [[ -z "${FIXTURE_UNTRACKED:-}" ]] || printf '?? %s\0' "$FIXTURE_UNTRACKED"
    exit 0 ;;
  diff)
    case "$2" in
      --binary) cat Cargo.toml Cargo.lock releases/query/CHANGELOG.md ;;
      --cached)
        if [[ "${3:-}" == --quiet ]]; then
          [[ "${FIXTURE_STAGED:-}" != yes ]]
        else
          [[ -z "${FIXTURE_CHANGED_PATH:-}" ]] || printf '%s\0' "$FIXTURE_CHANGED_PATH"
        fi ;;
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
        [[ "$#" == 6 && -f "commits/$4/parent" && "$5" == -m && "$6" == "Release query ${3#query-v}" ]] || exit 1
        mkdir -p tags
        printf '%s\n' "$4" > "tags/$3^{commit}"
        printf '%s\n' "${3#query-v}" > tag
        echo tag >> events ;;
      *) exit 2 ;;
    esac ;;
  cat-file) [[ -f tag ]] || exit 1; echo tag ;;
  ls-remote)
    [[ "${FIXTURE_REMOTE_FAIL:-}" != yes ]] || exit 47
    [[ "$2" == --refs && "$3" == -- && "$4" == https://example.invalid/ic-query ]] || exit 1
    for ref in "$@"; do
      case "$ref" in
        refs/heads/main) if [[ -f remote-head ]]; then printf '%s\t%s\n' "$(cat remote-head)" "$ref"; fi ;;
        refs/tags/*) if [[ -f remote-tag && "$(cat remote-tag-name)" == "$ref" ]]; then printf '%s\t%s\n' "$(cat remote-tag)" "$ref"; fi ;;
      esac
    done ;;
  add)
    [[ "$#" == 8 && "$2" == -- ]] || exit 1
    shift 2
    printf '%s\n' "$@" | sort > staged-paths
    printf '%s\n' Cargo.toml Cargo.lock releases/query/CHANGELOG.md crates/ic-query/Cargo.toml crates/ic-query-testkit/Cargo.toml "releases/query/$(sed 's/\.[0-9]*$//' candidate).md" | sort > expected-paths
    cmp staged-paths expected-paths
    mkdir -p index
    cp Cargo.toml Cargo.lock README.md releases/query/CHANGELOG.md index/
    cp -R docs ci crates releases index/
    echo stage >> events ;;
  commit)
    [[ "$#" == 3 && "$2" == -m && "$3" == "Release query $(cat candidate)" ]] || exit 1
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
      && "$7" == "refs/tags/query-v$(cat tag):refs/tags/query-v$(cat tag)" ]] || exit 1
    push_head="$(resolve "${6%:refs/heads/main}")"
    if [[ -f remote-head ]]; then ancestor "$(cat remote-head)" "$push_head"; fi
    printf '%s %s\n' "$push_head" "$(cat tag)" >> pushes
    echo push >> events
    [[ "${FIXTURE_BEFORE_PUSH:-}" != yes ]] || exit 47
    echo "$push_head" > remote-head
    echo "$tag_sha" > remote-tag
    printf 'refs/tags/query-v%s\n' "$(cat tag)" > remote-tag-name
    [[ "${FIXTURE_LOST_PUSH_REPLY:-}" != yes ]] || exit 47 ;;
  archive)
    selected="$(resolve "${!#}")"
    tar -cf - -C "commits/$selected/files" . ;;
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
if [[ "$*" != 'metadata --locked --offline --format-version 1' ]]; then printf 'cargo %s\n' "$*" >> events; fi
case "$*" in
  'fetch --locked'|'fetch --locked --offline') [[ "${FIXTURE_MISSING_DEPENDENCY:-}" != yes ]] || exit 43 ;;
  'sort --version'|'fmt --version')
    [[ -f selected-formatter && "${CARGO_NET_OFFLINE:-}" == true && "${RUSTUP_AUTO_INSTALL:-}" == 0 ]] || exit 1
    [[ -z "${IC_QUERY_RELEASE_PREPARE_CACHE+x}" ]] || exit 1
    if [[ "$1" == sort ]]; then
      . ci/tool-versions.env
      printf 'cargo-sort %s\n' "$SHARED_TOOLING_CARGO_SORT_VERSION"
    else
      echo rustfmt
    fi ;;
  'sort --workspace --grouped --check') [[ "${FIXTURE_MANIFEST_ORDER_FAILURE:-}" != yes ]] || exit 79 ;;
  'metadata --locked --offline --no-deps --format-version 1') printf '{}\n' ;;
  'metadata --locked --offline --format-version 1')
    packages='[]'
    for package in ic-query ic-query-testkit ic-query-cli ic-query-text; do
      version="$("$YQ" -p toml -o json -r '.package.version' "crates/$package/Cargo.toml")"
      packages="$(jq --arg name "$package" --arg version "$version" --arg path "$PWD/crates/$package/Cargo.toml" '. + [{id:$name,name:$name,version:$version,source:null,manifest_path:$path,publish:null,dependencies:[]}]' <<< "$packages")"
    done
    jq -n --arg root "$PWD" --argjson packages "$packages" '{workspace_root:$root,workspace_members:($packages|map(.id)),packages:$packages}' ;;

  generate-lockfile) sed "s/1.0.0/${FIXTURE_NEWER_DEPENDENCY:-2.0.0}/" Cargo.lock > changed.lock; mv changed.lock Cargo.lock ;;
  *) exit 2 ;;
esac
STUB
cat > "$work_dir/install-rust-tools.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ -z "${IC_QUERY_RELEASE_PREPARE_CACHE+x}" ]] || exit 1
. ci/tool-versions.env
editor="--consumer $PWD --package ic-build --lockfile Cargo.lock --bin ic-build-release-metadata --profile debug --features cli"
if [[ "$*" == "$editor" || "$*" == "$editor --check" ]]; then
  if [[ "$*" == *' --check' ]]; then
    [[ -f selected-editor && "${FIXTURE_EDITOR_CHECK_FAILURE:-}" != yes ]] || exit 53
  else
    echo 'editor setup' >> events
    [[ "${FIXTURE_EDITOR_SETUP_FAILURE:-}" != yes ]] || exit 52
    : > selected-editor
  fi
  printf '%s\n' "$CARGO_RELEASE_METADATA_EDITOR"
  exit
fi
for package in cargo-audit cargo-machete; do
  case "$package" in cargo-audit) version=0.22.2 ;; cargo-machete) version=0.9.2 ;; esac
  selected="--consumer $PWD --package $package --version $version --bin $package --profile release"
  if [[ "$*" == "$selected" || "$*" == "$selected --check" ]]; then
    if [[ "$*" == *' --check' ]]; then
      printf '%s check\n' "$package" >> events
      [[ -f "selected-$package" && "${FIXTURE_DEPENDENCY_CHECK_FAILURE:-}" != "$package" ]] || exit 54
    else
      printf '%s setup\n' "$package" >> events
      [[ "${FIXTURE_DEPENDENCY_SETUP_FAILURE:-}" != "$package" ]] || exit 55
      printf 'admitted %s\n' "$package" > "selected-$package"
    fi
    printf '/fixture/%s\n' "$package"
    exit
  fi
done
expected="--consumer $PWD --versions $PWD/ci/tool-versions.env"
if [[ "$*" == "$expected --check" ]]; then
  echo 'formatter check' >> events
  [[ -f selected-formatter && "${CARGO_NET_OFFLINE:-}" == true ]] || exit 1
  [[ "${FIXTURE_FORMAT_CHECK_FAILURE:-}" != yes ]] || exit 51
else
  [[ "$*" == "$expected" ]] || exit 1
  echo 'formatter setup' >> events
  [[ "${FIXTURE_FORMAT_SETUP_FAILURE:-}" != yes ]] || exit 49
  : > selected-formatter
fi
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
    [[ "${CARGO_NET_OFFLINE:-}" == true && "${CHANGELOG_VERSION:-}" == "$(cat candidate)" ]] || exit 1
    [[ -z "${IC_QUERY_RELEASE_PREPARE_CACHE+x}" ]] || exit 1
    [[ -f selected-formatter && -f selected-cargo-audit && -f selected-cargo-machete ]] || exit 1
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
  mkdir -p "$work_dir/$name/releases/query" "$work_dir/$name/scripts/ci" "$work_dir/$name/scripts/dev" "$work_dir/$name/scripts/release" "$work_dir/$name/ci" "$work_dir/$name/docs/changelog" "$work_dir/$name/target"
  cd "$work_dir/$name"
  bash "$repo_root/scripts/ci/prepare-script-fixture.sh" "$repo_root" "$PWD" \
    Makefile scripts/release/adapter.sh scripts/ci/check-changelog-version.sh scripts/ci/cargo-release-group.sh
  cp "$work_dir/install-rust-tools.sh" scripts/dev/
  # Runner recovery effects are simulated here; the real plan/custody boundary
  # is qualified separately by check-release-validation.sh.
  cat > scripts/release/validation.sh <<'STUB'
set -euo pipefail
case "$1" in
  run) make --no-print-directory ci ;;
  check) [[ -f "$2/passed" ]] ;;
  *) exit 2 ;;
esac
STUB
  candidate="$(bash scripts/ci/next-release-version.sh 0.46.5 "$kind")"
  minor="${candidate%.*}"
  printf '%s\n' "$candidate" > candidate
  printf 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n' > 'head'
  cat > Cargo.toml <<'MANIFEST'
[workspace]
members = ["crates/ic-query", "crates/ic-query-testkit", "crates/ic-query-cli", "crates/ic-query-text"]
[workspace.dependencies]
ic-query = { path = "crates/ic-query", version = "0.46.5", default-features = false }
ic-query-cli = { path = "crates/ic-query-cli", version = "0.46.5" }
ic-query-text = { path = "crates/ic-query-text", version = "0.46.5" }
ic-query-testkit = { path = "crates/ic-query-testkit", version = "0.46.5" }
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

[[package]]
name = "ic-query-testkit"
version = "0.46.5"
dependencies = ["ic-query"]
LOCK
  for package in ic-query ic-query-testkit ic-query-cli ic-query-text; do
    mkdir -p "crates/$package"
    printf '[package]\nname = "%s"\nversion = "0.46.5"\n' "$package" > "crates/$package/Cargo.toml"
  done
  mkdir -p releases/query releases/cli
  printf '# CLI history\n' > releases/cli/CHANGELOG.md
  printf '# Root changelog index\n' > CHANGELOG.md
  jq --arg detail "releases/query/$minor.md" '.groups |= map(if .name == "query" then .metadata_files = [$detail] else .metadata_files = [] end)' "$repo_root/ci/cargo-release-groups.json" > ci/cargo-release-groups.json
  printf 'ic-query = { version = "0.46", default-features = false }\n' > README.md
  cp README.md docs/library-usage.md
  printf "# Changelog\n\n## [%s]\n\n- Current batch.\n\n## [0.46.x] - 2026-10-04\n\n- \`0.46.5\` historical release.\n" "$candidate" > releases/query/CHANGELOG.md
  printf '# %s Changelog\n\n## %s\n\n- Current batch.\n' "$minor" "$candidate" > "releases/query/$minor.md"
  printf '\n[[package]]\nname = "ic-query-text"\nversion = "0.46.5"\n' >> Cargo.lock
  cp Cargo.lock original-lock
  mkdir -p commits/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/files
  cp Cargo.toml Cargo.lock README.md releases/query/CHANGELOG.md commits/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/files/
  cp -R docs ci crates releases commits/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/files/
  printf 'retained artifact\n' > target/original-artifact
}
run_release() {
  if [[ "$1" == resume ]]; then
    "$REAL_MAKE" --no-print-directory release-resume RELEASE_GROUP=query "VERSION=$2" > output 2>&1
  else
    "$REAL_MAKE" --no-print-directory "release-$1" RELEASE_GROUP=query > output 2>&1
  fi
}
expect_failure() { if run_release "$@"; then fail "unexpected successful $*"; fi; }
count_event() { awk -v event="$1" '$0 == event { count++ } END { print count+0 }' events; }
check_complete() {
  local candidate minor expected_lock
  candidate="$(cat candidate)"
  minor="${candidate%.*}"
  [[ "$(bash scripts/ci/cargo-release-group.sh version query)" == "$candidate" && "$(cat tag)" == "$candidate" ]] || exit 1
  [[ "$(count_event commit)" == 1 && "$(count_event tag)" == 1 && "$(count_event push)" == 1 ]] || exit 1
  [[ "$(sed -n '11p' ".release-state/query-$candidate.plan")" == complete && ! -e .release-state/lock ]] || exit 1
  [[ -f target/original-artifact && -f target/retained-output ]] || exit 1
  grep -Fxq "## [$candidate] - $(date -u +%F)" releases/query/CHANGELOG.md
  grep -Fxq "## $candidate - $(date -u +%F)" "releases/query/$minor.md"
  grep -Fq -- "- \`0.46.5\` historical release." releases/query/CHANGELOG.md
  export SELECTED_FIXTURE_VERSION="$candidate"
  expected_lock="$(perl -0pe 's/(name = "ic-query(?:-testkit)?"\nversion = ")0\.46\.5/$1$ENV{SELECTED_FIXTURE_VERSION}/g' original-lock)"
  [[ "$(cat Cargo.lock)" == "$expected_lock" ]] || fail 'dependency selection changed'
  bash scripts/ci/check-changelog-version.sh "$candidate"
}
commit_fix() {
  local kind="$1" previous candidate minor fix=eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
  previous="$(bash scripts/ci/cargo-release-group.sh version query)"
  candidate="$(bash scripts/ci/next-release-version.sh "$previous" "$kind")"
  minor="${candidate%.*}"
  printf '%s\n' "$candidate" > candidate
  printf '# Changelog\n\n## [%s]\n\n- Reviewed callback fix.\n\n' "$candidate" > pending-notes
  cat releases/query/CHANGELOG.md >> pending-notes
  mv pending-notes releases/query/CHANGELOG.md
  if [[ -f "releases/query/$minor.md" ]]; then
    printf '# %s Changelog\n\n## %s\n\n- Reviewed callback fix.\n\n' "$minor" "$candidate" > pending-notes
    cat "releases/query/$minor.md" >> pending-notes
    mv pending-notes "releases/query/$minor.md"
  else
    printf '# %s Changelog\n\n## %s\n\n- Reviewed callback fix.\n' "$minor" "$candidate" > "releases/query/$minor.md"
  fi
  jq --arg detail "releases/query/$minor.md" '.groups |= map(if .name == "query" then .metadata_files = [$detail] else . end)' ci/cargo-release-groups.json > policy-next
  mv policy-next ci/cargo-release-groups.json
  printf 'Reviewed later source change\n' >> README.md
  mkdir -p "commits/$fix/files"
  cp head "commits/$fix/parent"
  printf 'Fix release callback\n' > "commits/$fix/subject"
  printf '%s\n' "$fix" > "commits/$fix/tree"
  cp Cargo.toml Cargo.lock README.md releases/query/CHANGELOG.md "commits/$fix/files/"
  cp -R docs ci crates releases "commits/$fix/files/"
  printf '%s\n' "$fix" > 'head'
  : > descendant
}
save_old_evidence() {
  cp .release-state/query-0.47.0.plan original-plan
  cp .release-state/query-0.47.0.validation original-binding
  cp -R .release-state/query-0.47.0.verify.* original-evidence
  if [[ -f 'tags/query-v0.47.0^{commit}' ]]; then cp 'tags/query-v0.47.0^{commit}' original-tag; fi
}
check_old_evidence() {
  cmp original-plan .release-state/query-0.47.0.plan
  cmp original-binding .release-state/query-0.47.0.validation
  if [[ -f original-tag ]]; then cmp original-tag 'tags/query-v0.47.0^{commit}'; fi
  [[ "$(cat 'tags/query-v0.47.0^{commit}')" == bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb ]] || exit 1
  diff -r original-evidence .release-state/query-0.47.0.verify.*
  [[ "$(sed -n '11p' .release-state/query-0.47.0.plan)" == push ]] || exit 1
}
# The CI assertion probe invokes this same setup and completion check directly.
if [[ "${1:-}" == --assert-completion ]]; then
  new_fixture assertion-phase patch
  run_release patch
  check_complete
  fixture_complete=true
  exit 0
fi

for observation in increment-files increment-preflight source-initial source-prepared; do
  new_fixture "observation-$observation" patch
  operation=preflight
  head_failure_at=''
  diagnostic='fixture source observation failed'
  case "$observation" in
    increment-*)
      [[ "$observation" != increment-files ]] || operation=files
      mv scripts/ci/next-release-version.sh scripts/ci/next-release-version-real.sh
      cat > scripts/ci/next-release-version.sh <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
bash scripts/ci/next-release-version-real.sh "$@"
echo 'fixture increment observation failed' >&2
exit 23
STUB
      diagnostic='fixture increment observation failed' ;;
    source-initial) head_failure_at=1 ;;
    source-prepared) head_failure_at=2 ;;
  esac
  mkdir original-source
  cp Cargo.toml Cargo.lock README.md releases/query/CHANGELOG.md head target/original-artifact original-source/
  cp -R docs ci crates releases original-source/
  status=0
  FIXTURE_HEAD_FAILURE_AT="$head_failure_at" IC_QUERY_RELEASE_PREPARE_CACHE=1 \
    RELEASE_KIND=patch RELEASE_PREVIOUS=0.46.5 RELEASE_VERSION=0.46.6 \
    RELEASE_DATE=2026-10-07 RELEASE_SOURCE=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
    RELEASE_REMOTE=origin RELEASE_BRANCH=main \
    bash scripts/release/adapter.sh "$operation" > output 2>&1 || status=$?
  [[ "$status" == 23 ]] || fail "$observation did not preserve failed observation status: $status"
  grep -Fq "$diagnostic" output || fail "$observation lost observation diagnostics"
  for file in Cargo.toml Cargo.lock README.md releases/query/CHANGELOG.md head; do
    cmp "original-source/$file" "$file"
  done
  cmp original-source/original-artifact target/original-artifact
  diff -r original-source/docs docs
  [[ ! -e index && ! -e tag && ! -e pushes && ! -e target/retained-output ]] \
    || fail "$observation reached validation or release effects"
  if [[ "$observation" == source-prepared ]]; then
    printf '%s\n' 'cargo fetch --locked' 'formatter setup' 'cargo-audit setup' 'cargo-machete setup' 'editor setup' > expected
    cmp expected events || fail 'failed prepared source observation continued to offline admission'
  else
    [[ ! -e events && ! -e selected-formatter ]] || fail "$observation reached preparation"
  fi
  case "$observation" in increment-*) [[ ! -e .release-state ]] || exit 1 ;; esac
done

new_fixture unsupported-delivery patch
if RELEASE_DELIVERY=pr RELEASE_KIND=patch RELEASE_PREVIOUS=0.46.5 \
  RELEASE_VERSION=0.46.6 RELEASE_DATE=2026-10-07 \
  RELEASE_SOURCE=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
  RELEASE_REMOTE=origin RELEASE_BRANCH=main \
  bash scripts/release/adapter.sh preflight > output 2>&1; then
  fail 'unqualified release delivery admitted'
fi
[[ ! -e .release-state && ! -e events ]] || fail 'refused delivery changed release state'
for kind in patch minor major; do
  new_fixture "success-$kind" "$kind"
  FIXTURE_NEWER_DEPENDENCY=9.0.0 FIXTURE_CHANGED_PATH=releases/query/CHANGELOG.md run_release "$kind" \
    || { cat output; fail "$kind"; }
  check_complete
  awk '/^cargo metadata / || /^cargo sort --workspace --grouped --check$/ { next } { print }' events > observed
  printf '%s\n' 'cargo fetch --locked' 'formatter setup' 'cargo-audit setup' 'cargo-machete setup' 'editor setup' 'formatter check' \
    'cargo sort --version' 'cargo fmt --version' validate stage commit tag push > expected
  cmp expected observed
done
new_fixture prepared-payload-retry patch
FIXTURE_LOST_PREPARE_REPLY=yes expect_failure patch
cp Cargo.lock saved-lock
run_release resume "$(cat candidate)" || { cat output; fail 'prepared Query payload resume'; }
check_complete
cmp saved-lock Cargo.lock
[[ "$(count_event validate)" == 1 ]] || fail 'prepared Query payload required new validation'

# A committed fix supersedes old validation for the same unprepared candidate.
new_fixture committed-preparation-restart patch
FIXTURE_FAIL_TARGET=release-prepare-version expect_failure patch
old_candidate="$(cat candidate)"
old_evidence="$(cat ".release-state/query-$old_candidate.validation")"
cp ".release-state/query-$old_candidate.validation" saved-binding
cp -R "$old_evidence" saved-evidence
printf 'Reviewed preparation fix\n' >> README.md
fix=eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
mkdir -p "commits/$fix/files"
cp head "commits/$fix/parent"
cp Cargo.toml Cargo.lock README.md releases/query/CHANGELOG.md "commits/$fix/files/"
cp -R docs ci crates releases "commits/$fix/files/"
printf '%s\n' "$fix" > 'head'
for conflict in selections dirty; do
  cp "$old_evidence/identity" saved-identity
  if [[ "$conflict" == selections ]]; then
    sed '8s/main/other/' saved-identity > "$old_evidence/identity"
  else
    export FIXTURE_DIRTY=yes
  fi
  if RELEASE_KIND=patch RELEASE_PREVIOUS=0.46.5 RELEASE_VERSION="$old_candidate" \
    RELEASE_DATE="$(date -u +%F)" RELEASE_SOURCE="$fix" RELEASE_REMOTE=origin \
    RELEASE_BRANCH=main bash scripts/release/adapter.sh preflight > output 2>&1; then
    fail 'conflicting superseded receipt admitted'
  fi
  cmp saved-binding ".release-state/query-$old_candidate.validation"
  [[ "$(count_event validate)" == 1 ]] || fail 'conflicting receipt dispatched validation'
  cp saved-identity "$old_evidence/identity"
  unset FIXTURE_DIRTY
done
FIXTURE_GATE_FAILURE=yes expect_failure patch
set -- .release-state/query-"$old_candidate".validation-attempt.*/"$old_candidate".validation
[[ $# == 1 && -f "$1" ]] || fail 'superseded validation pointer not retained'
archived_binding="$1"
cmp saved-binding "$archived_binding"
diff -r saved-evidence "$old_evidence"
[[ ! -e ".release-state/query-$old_candidate.validation" && "$(count_event stage)" == 0 ]] \
  || fail 'failed revalidation admitted old evidence or staged metadata'
failed_evidence=(.release-state/query-"$old_candidate".verify.*/validation.log)
[[ ${#failed_evidence[@]} == 2 ]] || fail 'revalidation evidence missing'
for log in "${failed_evidence[@]}"; do
  [[ "$log" == "$old_evidence/validation.log" ]] || failed_log="$log"
done
cp "$failed_log" failed-revalidation
run_release patch || { cat output; fail 'committed preparation restart'; }
check_complete
[[ "$(count_event validate)" == 3 ]] || fail 'committed source did not receive fresh validation'
new_evidence="$(cat ".release-state/query-$old_candidate.validation")"
[[ "$new_evidence" != "$old_evidence" && "$(sed -n '6p' "$new_evidence/identity")" == "$fix" ]] \
  || fail 'fresh receipt did not bind the new source'
cmp saved-binding "$archived_binding"
diff -r saved-evidence "$old_evidence"
cmp failed-revalidation "$failed_log"

# One real Query adapter descendant keeps lockfile/changelog integration coverage.
# Shared test-release-runner.sh owns the full recovery phase/conflict matrix.
new_fixture descendant-patch-before-push minor
FIXTURE_BEFORE_PUSH=yes expect_failure minor
save_old_evidence
commit_fix patch
cp Cargo.lock original-lock
run_release patch || { cat output; fail 'descendant ordinary increment'; }
[[ "$(bash scripts/ci/cargo-release-group.sh version query)" == "$(cat candidate)" ]] || exit 1
[[ "$(count_event validate)" == 2 && "$(count_event commit)" == 2 && "$(count_event tag)" == 2 ]] || exit 1
[[ "$(sed -n '11p' ".release-state/query-$(cat candidate).plan")" == complete ]] || exit 1
SELECTED_FIXTURE_VERSION="$(cat candidate)" perl -0pe 's/(name = "ic-query(?:-testkit)?"\nversion = ")0\.47\.0/$1$ENV{SELECTED_FIXTURE_VERSION}/g' original-lock > expected-lock
cmp expected-lock Cargo.lock
next_evidence="$(cat ".release-state/query-$(cat candidate).validation")"
[[ "$(sed -n '6p' "$next_evidence/identity")" == eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee ]] || exit 1
bash scripts/ci/check-changelog-version.sh "$(cat candidate)"
check_old_evidence
[[ "$(count_event push)" == 2 && "$(sed -n '2p' pushes)" == "$(cat head) $(cat candidate)" ]] || exit 1
[[ -f target/original-artifact && -f target/retained-output && ! -e .release-state/lock ]] || exit 1
for notes in root detail; do
  new_fixture "candidate-mismatch-$notes" patch
  if [[ "$notes" == root ]]; then notes_path=releases/query/CHANGELOG.md; else notes_path=releases/query/0.46.md; fi
  printf '# Changelog\n\n## [9.9.9]\n' > "$notes_path"
  cp "$notes_path" "commits/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/files/$notes_path"
  cp "$notes_path" conflicting-notes
  expect_failure patch
  cmp conflicting-notes "$notes_path"
  cmp original-lock Cargo.lock
  [[ ! -e events && ! -e .release-state/query-0.46.6.plan && ! -e .release-state/query-0.46.6.validation ]] || exit 1
done
for failure in gate missing-dependency formatter-setup formatter-check dirty staged untracked unusual observation; do
  new_fixture "failure-$failure" patch
  case "$failure" in
    gate) FIXTURE_GATE_FAILURE=yes expect_failure patch ;;
    missing-dependency) FIXTURE_MISSING_DEPENDENCY=yes expect_failure patch ;;
    formatter-setup) FIXTURE_FORMAT_SETUP_FAILURE=yes expect_failure patch ;;
    formatter-check) FIXTURE_FORMAT_CHECK_FAILURE=yes expect_failure patch ;;
    dirty) FIXTURE_DIRTY=yes expect_failure patch ;;
    staged) FIXTURE_STAGED=yes expect_failure patch ;;
    untracked) FIXTURE_UNTRACKED=caller-owned.txt expect_failure patch ;;
    unusual) FIXTURE_UNTRACKED=$'caller-owned\nfile.txt' expect_failure patch ;;
    observation) FIXTURE_STATUS_FAIL=yes expect_failure patch ;;
  esac
  [[ "$(bash scripts/ci/cargo-release-group.sh version query)" == 0.46.5 && ! -e tag && ! -e remote-head ]] || exit 1
  cmp original-lock Cargo.lock
  [[ -f target/original-artifact && ! -e .release-state/query-0.46.6.plan && ! -e .release-state/lock ]] || exit 1
  case "$failure" in
    formatter-setup|formatter-check)
      [[ "$(count_event validate)" == 0 && ! -e .release-state/query-0.46.6.validation ]] || exit 1
      printf '%s\n' 'cargo fetch --locked' 'formatter setup' > expected
      if [[ "$failure" == formatter-check ]]; then printf '%s\n' 'cargo-audit setup' 'cargo-machete setup' 'editor setup' 'formatter check' >> expected; fi
      cmp expected events ;;
    dirty|staged|untracked|unusual|observation)
      [[ ! -e events && ! -e .release-state/query-0.46.6.validation ]] || exit 1
      case "$failure" in
        dirty) grep -Fq 'commit or restore worktree changes before starting a release' output ;;
        staged) grep -Fq 'commit or restore staged changes before starting a release' output ;;
        untracked|unusual) grep -Fq 'commit or remove untracked source before starting a release' output ;;
        observation)
          grep -Fq 'fixture Git status observation failed' output
          grep -Fq 'cannot inspect release-source status' output
          if grep -Fq 'uncommitted paths' output; then fail 'Git failure labelled dirty source'; fi ;;
      esac ;;
  esac
  if [[ "$failure" == gate ]]; then
    failed_logs=(.release-state/query-0.46.6.verify.*/validation.log)
    [[ -f "${failed_logs[0]}" ]] || fail 'failed gate log was discarded'
    run_release patch || { cat output; fail 'fresh gate retry'; }
    [[ -f "${failed_logs[0]}" && "$(count_event validate)" == 2 ]] || exit 1
    check_complete
  fi
done
for conflict in payload dependency identity retained-inputs unrelated; do
  new_fixture "conflict-$conflict" patch
  FIXTURE_LOST_PREPARE_REPLY=yes expect_failure patch
  case "$conflict" in
    payload) echo 'unvalidated edit' >> releases/query/CHANGELOG.md ;;
    dependency) sed 's/1.0.0/2.0.0/' Cargo.lock > changed; mv changed Cargo.lock ;;
    identity) echo invalid > .release-state/query-0.46.6.verify.*/identity ;;
    retained-inputs) echo 'changed evidence' >> .release-state/query-0.46.6.verify.*/before/Cargo.lock ;;
    unrelated) export FIXTURE_CHANGED_PATH=src/unrelated.rs ;;
  esac
  expect_failure resume "$(cat candidate)"
  [[ ! -e tag && ! -e remote-head && -f target/retained-output ]] || exit 1
  unset FIXTURE_CHANGED_PATH
done
echo 'IC Query release adapter fixtures passed'

# Exercise the shared observation on inherited history without new commits.
source_context="$work_dir/source-context"
PATH="$fixture_native_path" "$REAL_GIT" clone --quiet --shared --no-checkout "$repo_root" "$source_context"
(
  export PATH="$fixture_native_path"
  cd "$source_context"
  git read-tree HEAD
  git checkout-index --all
  check_source() { bash "$repo_root/scripts/ci/check-release-source.sh" > "$work_dir/source-output" 2>&1; }
  refuse_source() { if check_source; then fail 'changed source admitted'; fi; }
  check_source
  cp .git/index "$work_dir/source-index"
  printf '\n' >> Cargo.lock
  cp Cargo.lock "$work_dir/source-lock"
  refuse_source
  grep -Fq 'unstaged: Cargo.lock' "$work_dir/source-output"
  cmp .git/index "$work_dir/source-index"
  cmp Cargo.lock "$work_dir/source-lock"
  git checkout-index --force -- Cargo.lock

  replacement="$(git rev-parse HEAD:AGENTS.md)"
  git update-index --cacheinfo "100644,$replacement,README.md"
  git diff --quiet HEAD -- README.md
  unusual=$'caller-owned\nfile.txt'
  printf 'caller evidence\n' > "$unusual"
  cp .git/index "$work_dir/source-index"
  refuse_source
  grep -Fq 'staged: README.md' "$work_dir/source-output"
  grep -Fq 'unstaged: README.md' "$work_dir/source-output"
  printf '  untracked: %q\n' "$unusual" > "$work_dir/source-expected"
  grep -Fx -f "$work_dir/source-expected" "$work_dir/source-output" > /dev/null
  cmp .git/index "$work_dir/source-index"
  [[ "$(cat "$unusual")" == 'caller evidence' ]] || exit 1
)
echo 'IC Query real-Git source diagnostics and preservation fixtures passed'

# A distinct parent checkout catches leaked logger identity without running CI.
metadata_context="$work_dir/metadata-context"
mkdir -p "$metadata_context"
cat > "$metadata_context/Makefile" <<'MAKE'
.PHONY: metadata-check ci
metadata-check:
	@RELEASE_GROUP=query bash "$(FIXTURE_REPOSITORY_ROOT)/scripts/ci/check-release-metadata.sh"
	@RELEASE_GROUP=cli bash "$(FIXTURE_REPOSITORY_ROOT)/scripts/ci/check-release-metadata.sh"
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
fixture_complete=true
