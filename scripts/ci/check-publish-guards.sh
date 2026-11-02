#!/usr/bin/env bash
set -euo pipefail
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
fail() { echo "publication fixture failed: $*" >&2; exit 1; }
publish_case="$work_dir/publish"
mkdir -p "$publish_case/bin" "$publish_case/source"
bash "$repo_root/scripts/ci/prepare-script-fixture.sh" "$repo_root" "$publish_case/source" \
  Makefile scripts/release/publish-workspace.sh scripts/release/check-tag-at-head.sh scripts/ci/check-crates-io-version.sh scripts/ci/run-cargo-publication.sh scripts/ci/check-release-source.sh scripts/ci/check-release-tag.sh
printf '[workspace]\n' > "$publish_case/source/Cargo.toml"
# Group graph admission is qualified with real Cargo by the metadata fixtures.
# This suite owns publication effects and supplies only its selected versions.
cat > "$publish_case/source/scripts/ci/cargo-release-group.sh" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == version ]] || exit 2
case "${2:-}" in query) echo 0.86.0 ;; cli) echo 0.85.1 ;; *) exit 2 ;; esac
STUB
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

chmod +x "$publish_case/bin/"*
export PATH="$publish_case/bin:$PATH"
new_case() {
  local name="$1"
  mkdir -p "$work_dir/$name/state"
  cp -R "$publish_case/source/." "$work_dir/$name/"
  cd "$work_dir/$name"
  export STATE_DIR="$PWD/state" TRACE_FILE="$PWD/trace"
  export CARGO_PUBLISH_INDEX_ATTEMPTS=2 CARGO_PUBLISH_INDEX_DELAY_SECONDS=0
}
run_group() { RELEASE_GROUP="$1" bash scripts/release/publish-workspace.sh > output 2>&1; }
refuse_group() { if run_group "$1"; then fail "unexpected successful $1 publication"; fi; }
receipt_check() {
  local package="$1" version="$2" receipt
  local receipts=("target/publication/$package."*)
  [[ ${#receipts[@]} == 1 ]] || fail "one receipt required for $package"
  receipt="${receipts[0]}"
  printf '%s\n' publish 1111111111111111111111111111111111111111 crates-io "$package@$version" > expected-intent
  cmp expected-intent "$receipt/intent"
  printf '%s\0' cargo publish --locked --registry crates-io --package "$package@$version" > expected-arguments
  cmp expected-arguments "$receipt/arguments"
  [[ "$(cat "$receipt/status")" == 0 && "$(cat "$receipt/cargo-status")" == 0 && "$(cat "$receipt/log-status")" == 0 ]] || fail 'lost successful statuses'
  grep -Fq "publication fixture stdout: $package" "$receipt/cargo.log"
  grep -Fq "publication fixture stderr: $package" "$receipt/cargo.log"
}
new_case query-only
run_group query
[[ -e state/ic-query && ! -e state/ic-query-text && ! -e state/ic-query-cli ]] || fail 'Query publication crossed group boundary'
receipt_check ic-query 0.86.0
cp trace first-trace
run_group query
[[ $(rg -c '^cargo publish' trace) == 1 ]] || fail 'published Query uploaded twice'
new_case cli-only
: > state/ic-query
run_group cli
[[ -e state/ic-query-text && -e state/ic-query-cli ]] || fail 'presentation publication incomplete'
printf '%s\n' \
  'observe ic-query@0.86.0' 'observe ic-query-text@0.85.1' \
  'cargo info ic-query@0.86.0 --registry crates-io' \
  'cargo publish --locked --registry crates-io --package ic-query-text@0.85.1' \
  'observe ic-query-cli@0.85.1' 'cargo info ic-query-text@0.85.1 --registry crates-io' \
  'cargo publish --locked --registry crates-io --package ic-query-cli@0.85.1' > expected
cmp expected trace
receipt_check ic-query-text 0.85.1
receipt_check ic-query-cli 0.85.1
run_group cli
[[ $(rg -c '^cargo publish' trace) == 2 ]] || fail 'presentation retry repeated uploads'
for selection in '' unknown; do
  new_case "unknown-${selection:-missing}"
  refuse_group "$selection"
  [[ ! -e trace && ! -e target ]] || fail 'invalid group reached publication'
done
new_case missing-query
refuse_group cli
[[ ! -e state/ic-query-text && ! -e state/ic-query-cli ]] || fail 'presentation preceded Query peer'
for package in ic-query ic-query-text ic-query-cli; do
  new_case "failed-$package"
  group=query
  if [[ "$package" != ic-query ]]; then group=cli; : > state/ic-query; fi
  status=0
  PUBLISH_ERROR_PACKAGE="$package" PUBLISH_ERROR_STATUS=47 run_group "$group" || status=$?
  [[ "$status" == 47 && ! -e state/$package ]] || fail 'Cargo failure lost status or became successful'
  [[ "$package" == ic-query-cli || ! -e state/ic-query-cli ]] || fail 'CLI published after prerequisite failed'
  unset PUBLISH_ERROR_PACKAGE PUBLISH_ERROR_STATUS
  run_group "$group"
  [[ -e state/$package ]] || fail 'known failure did not resume'
done
for package in ic-query ic-query-text ic-query-cli; do
  for failure in 429 503 transport; do
    new_case "registry-$package-$failure"
    group=query
    if [[ "$package" != ic-query ]]; then group=cli; : > state/ic-query; fi
    status=0
    if [[ "$failure" == transport ]]; then
      REGISTRY_ERROR_PACKAGE="$package" REGISTRY_TRANSPORT=6 run_group "$group" || status=$?
    else
      REGISTRY_ERROR_PACKAGE="$package" REGISTRY_HTTP="$failure" run_group "$group" || status=$?
    fi
    [[ "$status" != 0 && ! -e state/$package && ! -e state/ic-query-cli ]] || fail 'unavailable registry admitted an upload'
  done
done
new_case hidden-query-index
: > state/ic-query
HIDE_LIBRARY_INFO=1 refuse_group cli
[[ ! -e state/ic-query-text ]] || fail 'renderer uploaded before its peer was visible'
for refusal in dirty race; do
  new_case "$refusal"
  status=0
  if [[ "$refusal" == dirty ]]; then PUBLICATION_DIRTY=1 run_group query || status=$?
  else PUBLICATION_RACE=1 run_group query || status=$?; fi
  [[ "$status" != 0 && ! -e state/ic-query ]] || fail 'publication source race admitted'
  if [[ "$refusal" == dirty ]]; then
    [[ ! -e target/publication/.workspace-publication.lock && ! -e target/publication/.cargo-publication.lock ]] || fail 'pre-dispatch refusal retained locks'
  else
    [[ -e target/publication/.cargo-publication.lock/owner ]] || fail 'race lost dispatch-admission evidence'
  fi
done
new_case signal
status=0
PUBLICATION_SIGNAL=1 run_group query || status=$?
[[ "$status" == 143 ]] || fail 'signal status was lost'
[[ -e target/publication/.workspace-publication.lock/owner && -e target/publication/.cargo-publication.lock/owner ]] || fail 'uncertain upload lost locks'
cp trace before-retry
refuse_group query
cmp before-retry trace
# Actual Make routing and annotated selected-group tag admission.
new_case make-publication
: > state/ic-query
: > state/ic-query-text
: > state/ic-query-cli
cat > "$publish_case/bin/git" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
sha=1111111111111111111111111111111111111111
case "$*" in
  'rev-parse --show-prefix') printf '\n' ;;
  'rev-parse --show-toplevel') pwd -P ;;
  'status --porcelain --untracked-files=all'|'status --porcelain=v1 -z --untracked-files=all') [[ "${DIRTY_STATUS:-0}" == 0 ]] || printf ' M Cargo.lock\0' ;;
  'cat-file -t refs/tags/cli-v0.85.1') [[ "${MISSING_TAG:-0}" == 0 ]] || exit 1; echo "${TAG_TYPE:-tag}" ;;
  'rev-parse --verify refs/tags/cli-v0.85.1^{commit}') echo "${TAG_COMMIT:-$sha}" ;;
  'rev-parse --verify HEAD'|"rev-parse --verify $sha^{commit}") echo "$sha" ;;
  *) exit 2 ;;
esac
STUB
chmod +x "$publish_case/bin/git"
RELEASE_GROUP=cli make --no-print-directory publish > output 2>&1
[[ $(rg -c '^cargo clippy' trace) == 2 ]] || fail 'Make skipped publication lint admission'
for failure in dirty missing lightweight wrong-commit lint; do
  cp trace before-refusal
  status=0
  case "$failure" in
    dirty) DIRTY_STATUS=1 RELEASE_GROUP=cli make --no-print-directory publish > output 2>&1 || status=$? ;;
    missing) MISSING_TAG=1 RELEASE_GROUP=cli make --no-print-directory publish > output 2>&1 || status=$? ;;
    lightweight) TAG_TYPE=commit RELEASE_GROUP=cli make --no-print-directory publish > output 2>&1 || status=$? ;;
    wrong-commit) TAG_COMMIT=2222222222222222222222222222222222222222 RELEASE_GROUP=cli make --no-print-directory publish > output 2>&1 || status=$? ;;
    lint) PUBLISH_LINT_FAIL=-p RELEASE_GROUP=cli make --no-print-directory publish > output 2>&1 || status=$? ;;
  esac
  [[ "$status" != 0 ]] || fail "Make admitted $failure"
  [[ "$failure" == lint ]] || cmp before-refusal trace
done
echo 'Selected-group publication, receipts, recovery and Make admission fixtures passed'
fixture_complete=true
