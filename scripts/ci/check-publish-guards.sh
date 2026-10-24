#!/usr/bin/env bash
set -euo pipefail

# Fixture Make invocations own their selections, independently of the caller.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES

repo_root="$(CDPATH="" cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
export REAL_CARGO YQ
REAL_CARGO="$(command -v cargo)"
YQ="${YQ:-$repo_root/.tools/host/bin/yq}"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-publish-guards.XXXXXX")"
work_dir="$(CDPATH="" cd -P -- "$work_dir" && printf '%s/.' "$PWD")"
work_dir="${work_dir%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "Publish guard fixtures retained"' EXIT
export TMPDIR="${work_dir%/*}"

fail() {
  echo "error: $*" >&2
  exit 1
}

publish_case="${work_dir}/publish"
mkdir -p "${publish_case}/bin" "${publish_case}/state" "${publish_case}/source/scripts/ci" "${publish_case}/source/scripts/release"
current_version="$(bash "$repo_root/scripts/ci/read-cargo-workspace-version.sh" --stable "$repo_root/Cargo.toml")"
# Keep Cargo validation real without referencing absent production members.
printf '[workspace.package]\nversion = "%s"\n' \
  "$current_version" > "$publish_case/source/Cargo.toml"
cp "$repo_root/scripts/ci/"{check-crates-io-version.sh,read-cargo-workspace-version.sh,run-cargo-publication.sh,check-make-execution.sh} "$publish_case/source/scripts/ci/"
cp "$repo_root/scripts/release/publish-workspace.sh" "$publish_case/source/scripts/release/"
cat > "$publish_case/bin/git" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
  'rev-parse --show-toplevel') pwd -P ;;
  'rev-parse --verify HEAD') printf '%s\n' 1111111111111111111111111111111111111111 ;;
  'status --porcelain --untracked-files=all')
    if [[ -n "${PUBLICATION_DIRTY:-}" || ( "${PUBLICATION_RACE:-0}" == 1 && -d target/publication/.cargo-publication.lock ) ]]; then
      printf ' M Cargo.lock\n'
    fi
    ;;
  *) exit 2 ;;
esac
EOF
cat > "${publish_case}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == locate-project ]]; then exec "$REAL_CARGO" "$@"; fi
printf 'cargo %s\n' "$*" >> "${TRACE_FILE}"
case "${1:-}" in
  clippy)
    [[ "${PUBLISH_LINT_FAIL:-}" != "$2" ]] || exit 61
    ;;
  info)
    package="${2%@*}"
    if [[ "${package}" == "ic-query" && -n "${HIDE_LIBRARY_INFO:-}" ]]; then
      exit 1
    fi
    [[ -e "${STATE_DIR}/${package}" ]] || exit 1
    ;;
  publish)
    package=""
    while [[ "$#" -gt 0 ]]; do
      if [[ "$1" == "--package" ]]; then
        package="${2%@*}"
        break
      fi
      shift
    done
    [[ -n "${package}" ]] || exit 2
    printf 'publication fixture stdout: %s\n' "$package"
    printf 'publication fixture stderr: %s\n' "$package" >&2
    if [[ "${PUBLICATION_SIGNAL:-0}" == 1 ]]; then kill -TERM "$PPID"; fi
    if [[ "${package}" == "${PUBLISH_ERROR_PACKAGE:-}" ]]; then
      exit "${PUBLISH_ERROR_STATUS:-47}"
    fi
    : > "${STATE_DIR}/${package}"
    ;;
  *)
    exit 2
    ;;
esac
EOF
cat > "${publish_case}/bin/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
url="${!#}"
version="${url##*/}"
package="${url%/*}"
package="${package##*/}"
printf 'observe %s@%s\n' "$package" "$version" >> "$TRACE_FILE"
if [[ "$package" == "${REGISTRY_ERROR_PACKAGE:-}" ]]; then
  printf '%s' "${REGISTRY_HTTP:-503}"
  exit "${REGISTRY_TRANSPORT:-0}"
fi
if [[ -e "$STATE_DIR/$package" ]]; then printf 200; else printf 404; fi
EOF
chmod +x "${publish_case}/bin/"*
(
  cd "$publish_case/source"
  PATH="${publish_case}/bin:${PATH}" TRACE_FILE="${publish_case}/trace" \
    STATE_DIR="${publish_case}/state" CARGO_PUBLISH_INDEX_ATTEMPTS=2 \
    CARGO_PUBLISH_INDEX_DELAY_SECONDS=0 \
    bash scripts/release/publish-workspace.sh
) >/dev/null
printf '%s\n' \
  "observe ic-query@${current_version}" \
  "cargo publish --locked --registry crates-io --package ic-query@${current_version}" \
  "observe ic-query-cli@${current_version}" \
  "cargo info ic-query@${current_version} --registry crates-io" \
  "cargo publish --locked --registry crates-io --package ic-query-cli@${current_version}" \
  > "${publish_case}/expected-trace"
cmp -s "${publish_case}/expected-trace" "${publish_case}/trace" \
  || fail "the workspace publisher ran an unexpected Cargo command sequence"

for package in ic-query ic-query-cli; do
  receipts=("$publish_case/source/target/publication/$package."*)
  [[ "${#receipts[@]}" == 1 ]] || fail "expected one retained $package publication receipt"
  receipt="${receipts[0]}"
  printf '%s\n' publish 1111111111111111111111111111111111111111 crates-io "$package@$current_version" > "$publish_case/expected-intent"
  cmp -s "$publish_case/expected-intent" "$receipt/intent" || fail 'publication intent lost package/source binding'
  printf '%s\0' cargo publish --locked --registry crates-io --package "$package@$current_version" > "$publish_case/expected-arguments"
  cmp -s "$publish_case/expected-arguments" "$receipt/arguments" || fail 'publication arguments were not retained exactly'
  [[ "$(cat "$receipt/status")" == 0 && "$(cat "$receipt/cargo-status")" == 0 && "$(cat "$receipt/log-status")" == 0 ]] \
    || fail 'successful publication receipt lost completion status'
  grep -Fq "publication fixture stdout: $package" "$receipt/cargo.log" || fail 'publication stdout missing'
  grep -Fq "publication fixture stderr: $package" "$receipt/cargo.log" || fail 'publication stderr missing'
done
[[ ! -e "$publish_case/source/target/publication/.workspace-publication.lock" && ! -e "$publish_case/source/target/publication/.cargo-publication.lock" ]] \
  || fail 'completed publication retained an admission lock'

: > "${publish_case}/trace"
(
  cd "$publish_case/source"
  PATH="${publish_case}/bin:${PATH}" TRACE_FILE="${publish_case}/trace" \
    STATE_DIR="${publish_case}/state" CARGO_PUBLISH_INDEX_ATTEMPTS=2 \
    CARGO_PUBLISH_INDEX_DELAY_SECONDS=0 \
    bash scripts/release/publish-workspace.sh
) >/dev/null
printf '%s\n' \
  "observe ic-query@${current_version}" \
  "observe ic-query-cli@${current_version}" \
  > "${publish_case}/expected-trace"
cmp -s "${publish_case}/expected-trace" "${publish_case}/trace" \
  || fail "the workspace publisher was not retry-safe for published crates"

for package in ic-query ic-query-cli; do
  state="$publish_case/publish-failure-$package"
  mkdir -p "$state"
  failed_publish_status=0
  (
    cd "$publish_case/source"
    PATH="$publish_case/bin:$PATH" TRACE_FILE="$state/trace" STATE_DIR="$state" \
      PUBLISH_ERROR_PACKAGE="$package" PUBLISH_ERROR_STATUS=47 \
      CARGO_PUBLISH_INDEX_ATTEMPTS=2 CARGO_PUBLISH_INDEX_DELAY_SECONDS=0 \
      bash scripts/release/publish-workspace.sh
  ) > "$state/output" 2>&1 || failed_publish_status=$?
  [[ "$failed_publish_status" == 47 ]] \
    || fail "the workspace publisher lost the $package publish failure"
  [[ ! -e "$state/ic-query-cli" ]] || fail 'the CLI was published after a Cargo failure'
  if [[ "$package" == ic-query ]]; then
    [[ ! -e "$state/ic-query" ]] || fail 'the failed library publish was treated as successful'
  else
    [[ -e "$state/ic-query" ]] || fail 'the CLI publish preceded the library'
  fi
done

mkdir -p "${publish_case}/hidden-index-state"
for index_attempt in initial retry; do
  if (
    cd "$publish_case/source"
    PATH="${publish_case}/bin:${PATH}" TRACE_FILE="${publish_case}/hidden-index-trace" \
      STATE_DIR="${publish_case}/hidden-index-state" HIDE_LIBRARY_INFO=1 \
      CARGO_PUBLISH_INDEX_ATTEMPTS=2 CARGO_PUBLISH_INDEX_DELAY_SECONDS=0 \
      bash scripts/release/publish-workspace.sh
  ) >/dev/null 2>&1; then
    hidden_index_status=0
  else
    hidden_index_status="$?"
  fi
  [[ "${hidden_index_status}" -ne 0 ]] \
    || fail "the workspace publisher accepted an unindexed library on $index_attempt"
  [[ -e "${publish_case}/hidden-index-state/ic-query" ]] \
    || fail "the workspace publisher did not publish the missing library"
  [[ ! -e "${publish_case}/hidden-index-state/ic-query-cli" ]] \
    || fail "the workspace publisher published the CLI before the library was indexed"
done

for package in ic-query ic-query-cli; do
  for failure in http transport; do
    state="$publish_case/unavailable-$package-$failure"
    mkdir -p "$state"
    : > "$state/ic-query"
    transport=0
    http=503
    if [[ "$failure" == transport ]]; then transport=28; http=404; fi
    status=0
    (
      cd "$publish_case/source"
      PATH="$publish_case/bin:$PATH" TRACE_FILE="$state/trace" STATE_DIR="$state" \
        REGISTRY_ERROR_PACKAGE="$package" REGISTRY_HTTP="$http" \
        REGISTRY_TRANSPORT="$transport" CARGO_PUBLISH_INDEX_ATTEMPTS=2 \
        CARGO_PUBLISH_INDEX_DELAY_SECONDS=0 bash scripts/release/publish-workspace.sh
    ) > "$state/output" 2>&1 || status=$?
    [[ "$status" == 2 ]] || fail 'unavailable registry observation did not stop publication'
    printf '%s\n' "observe ic-query@$current_version" > "$state/expected"
    if [[ "$package" == ic-query-cli ]]; then
      printf '%s\n' "observe ic-query-cli@$current_version" >> "$state/expected"
    fi
    cmp -s "$state/expected" "$state/trace" \
      || fail 'unavailable registry observation was retried or reached publication'
    [[ ! -e "$state/ic-query-cli" ]] || fail 'unavailable observation published the CLI'
  done
done

for condition in dirty raced; do
  refused="$publish_case/source-$condition"
  mkdir -p "$refused/state"
  cp -R "$publish_case/source/scripts" "$refused/"
  cp "$publish_case/source/Cargo.toml" "$refused/"
  status=0
  (
    cd "$refused"
    case "$condition" in dirty) export PUBLICATION_DIRTY=1 ;; raced) export PUBLICATION_RACE=1 ;; esac
    PATH="$publish_case/bin:$PATH" TRACE_FILE="$refused/trace" STATE_DIR="$refused/state" \
      bash scripts/release/publish-workspace.sh
  ) > "$refused/output" 2>&1 || status=$?
  [[ "$status" == 2 ]] || fail 'source admission failure lost its status'
  printf '%s\n' "observe ic-query@$current_version" > "$refused/expected-trace"
  cmp -s "$refused/expected-trace" "$refused/trace" || fail 'source admission failure reached Cargo'
  if [[ "$condition" == raced ]]; then
    receipts=("$refused/target/publication/ic-query."*)
    [[ -s "${receipts[0]}/intent" && ! -e "${receipts[0]}/dispatch" && -e "$refused/target/publication/.workspace-publication.lock/owner" ]] \
      || fail 'pre-dispatch source race discarded retained intent or admission lock'
  fi
done

# An uncertain dispatch keeps both locks; a later invocation must not observe or upload.
interrupted="$publish_case/interrupted"
mkdir -p "$interrupted/state"
cp -R "$publish_case/source/scripts" "$interrupted/"
cp "$publish_case/source/Cargo.toml" "$interrupted/"
status=0
(
  cd "$interrupted"
  PATH="$publish_case/bin:$PATH" TRACE_FILE="$interrupted/trace" STATE_DIR="$interrupted/state" \
    PUBLICATION_SIGNAL=1 bash scripts/release/publish-workspace.sh
) > "$interrupted/output" 2>&1 || status=$?
[[ "$status" == 143 ]] || fail 'interrupted upload lost its signal status'
receipts=("$interrupted/target/publication/ic-query."*)
receipt="${receipts[0]}"
[[ -s "$receipt/dispatch" && ! -e "$receipt/status" && -e "$interrupted/target/publication/.cargo-publication.lock/owner" && -e "$interrupted/target/publication/.workspace-publication.lock/owner" ]] \
  || fail 'uncertain upload discarded dispatch evidence or admission locks'
cp "$interrupted/trace" "$interrupted/before-trace"
status=0
(
  cd "$interrupted"
  PATH="$publish_case/bin:$PATH" TRACE_FILE="$interrupted/trace" STATE_DIR="$interrupted/state" \
    bash scripts/release/publish-workspace.sh
) > "$interrupted/retry-output" 2>&1 || status=$?
[[ "$status" != 0 ]] || fail 'uncertain publication was replayed'
cmp -s "$interrupted/before-trace" "$interrupted/trace" || fail 'uncertain publication reached observation or Cargo'

make_case="${work_dir}/make-publish"
mkdir -p "${make_case}/bin" "${make_case}/scripts/release" "${make_case}/scripts/ci"
printf '[fixture]\nversion = "9.9.9"\n[workspace.package]\nversion = "%s"\n' \
  "${current_version}" > "${make_case}/Cargo.toml"
cp "${repo_root}/scripts/ci/"{check-crates-io-version.sh,read-cargo-workspace-version.sh,run-cargo-publication.sh,check-make-execution.sh} "${make_case}/scripts/ci/"
ln -s "${repo_root}/scripts/release/publish-workspace.sh" \
  "${make_case}/scripts/release/publish-workspace.sh"
cat > "${make_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
head_commit=1111111111111111111111111111111111111111
case "$*" in
  'rev-parse --show-prefix') printf '\n' ;;
  'rev-parse --show-toplevel') pwd -P ;;
  'status --porcelain --untracked-files=all') exit 0 ;;
  'status --porcelain=v1 -z --untracked-files=all')
    [[ "${DIRTY_STATUS:-0}" == 0 ]] || printf ' M Cargo.lock\0'
    [[ -z "${UNTRACKED_PATH:-}" ]] || printf '?? %s\0' "${UNTRACKED_PATH}"
    exit "${INVENTORY_STATUS:-0}"
    ;;
  "cat-file -t refs/tags/v${RELEASE_VERSION}")
    [[ -z "${MISSING_TAG:-}" ]] || exit 1
    printf '%s\n' "${TAG_TYPE:-tag}"
    ;;
  "rev-parse --verify refs/tags/v${RELEASE_VERSION}^{commit}")
    printf '%s\n' "${TAG_COMMIT:-$head_commit}"
    ;;
  'rev-parse --verify HEAD') printf '%s\n' "$head_commit" ;;
  "rev-parse --verify $head_commit^{commit}") printf '%s\n' "$head_commit" ;;
  *) exit 2 ;;
esac
EOF
chmod +x "${make_case}/bin/git"

# Already-published fixtures exercise Make's entry checks without registry IO.
[[ "$(cd "${make_case}" && make --no-print-directory -s -f "${repo_root}/Makefile" version)" == "${current_version}" ]] \
  || fail "make version did not select the canonical workspace version"
(
  cd "${make_case}"
  PATH="${make_case}/bin:${publish_case}/bin:${PATH}" RELEASE_VERSION="${current_version}" \
    TRACE_FILE="${make_case}/trace" STATE_DIR="${publish_case}/state" \
    make --no-print-directory -f "${repo_root}/Makefile" publish
) >/dev/null \
  || fail "make publish rejected a clean tagged release"
printf '%s\n' \
  'cargo clippy -p ic-query --all-targets --no-default-features --locked --offline -- -D warnings' \
  'cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings' \
  > "$make_case/expected-trace"
cat "$publish_case/expected-trace" >> "$make_case/expected-trace"
cmp -s "${make_case}/expected-trace" "${make_case}/trace" \
  || fail "make publish did not delegate once to the retry-safe workspace publisher"

for lint_scope in -p --workspace; do
  : > "$make_case/lint-trace"
  if (
    cd "$make_case"
    PATH="$make_case/bin:$publish_case/bin:$PATH" RELEASE_VERSION="$current_version" \
      TRACE_FILE="$make_case/lint-trace" STATE_DIR="$publish_case/state" \
      PUBLISH_LINT_FAIL="$lint_scope" \
      make --no-print-directory -j4 -f "$repo_root/Makefile" publish
  ) > "$make_case/lint-output" 2>&1; then
    fail 'make publish ignored failed Clippy gate'
  fi
  printf '%s\n' \
    'cargo clippy -p ic-query --all-targets --no-default-features --locked --offline -- -D warnings' \
    > "$make_case/expected-lint-trace"
  if [[ "$lint_scope" == --workspace ]]; then
    printf '%s\n' 'cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings' \
      >> "$make_case/expected-lint-trace"
  fi
  cmp "$make_case/lint-trace" "$make_case/expected-lint-trace" \
    || fail 'failed Clippy reached publication or registry observation'
done

mkdir "$make_case/upload-state"
(
  cd "$make_case"
  PATH="$make_case/bin:$publish_case/bin:$PATH" RELEASE_VERSION="$current_version" \
    TRACE_FILE="$make_case/upload-trace" STATE_DIR="$make_case/upload-state" \
    CARGO_PUBLISH_INDEX_ATTEMPTS=2 CARGO_PUBLISH_INDEX_DELAY_SECONDS=0 \
    make --no-print-directory -f "$repo_root/Makefile" publish
) > "$make_case/upload-output" 2>&1 || fail 'make publish rejected a clean tagged missing-package selection'
[[ -e "$make_case/upload-state/ic-query" && -e "$make_case/upload-state/ic-query-cli" ]] \
  || fail 'make publish did not dispatch both admitted packages'
for package in ic-query ic-query-cli; do
  receipts=("$make_case/target/publication/$package."*)
  [[ "${#receipts[@]}" == 1 && "$(cat "${receipts[0]}/status")" == 0 ]] \
    || fail 'make publish did not retain the completed upload receipt'
done

for invalid_release in dirty untracked inventory-empty inventory-partial stale-tag missing-tag lightweight-tag; do
  : > "${make_case}/trace"
  if (
    cd "${make_case}"
    export PATH="${make_case}/bin:${publish_case}/bin:${PATH}"
    export RELEASE_VERSION="${current_version}" TRACE_FILE="${make_case}/trace"
    export STATE_DIR="${publish_case}/state"
    case "${invalid_release}" in
      dirty) export DIRTY_STATUS=1 ;;
      untracked) export UNTRACKED_PATH=unexpected.txt ;;
      inventory-empty) export INVENTORY_STATUS=9 ;;
      inventory-partial) export INVENTORY_STATUS=9 UNTRACKED_PATH=unexpected.txt ;;
      stale-tag) export TAG_COMMIT=2222222222222222222222222222222222222222 ;;
      missing-tag) export MISSING_TAG=1 ;;
      lightweight-tag) export TAG_TYPE=commit ;;
    esac
    make --no-print-directory -f "${repo_root}/Makefile" publish
  ) > "$make_case/$invalid_release.log" 2>&1; then
    fail "make publish accepted a ${invalid_release} release"
  fi
  [[ ! -s "${make_case}/trace" ]] \
    || fail "make publish reached the registry or Cargo for a ${invalid_release} release"
  case "$invalid_release" in
    dirty) grep -Fq 'unstaged: Cargo.lock' "$make_case/$invalid_release.log" ;;
    untracked) grep -Fq 'untracked: unexpected.txt' "$make_case/$invalid_release.log" ;;
    inventory-*) grep -Fq 'cannot inspect release-source status' "$make_case/$invalid_release.log" ;;
  esac
done
fixture_complete=true
