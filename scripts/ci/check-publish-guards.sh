#!/usr/bin/env bash
set -euo pipefail

# Fixture Make invocations own their selections, independently of the caller.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export REAL_CARGO YQ
REAL_CARGO="$(command -v cargo)"
YQ="${YQ:-$repo_root/.tools/host/bin/yq}"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-publish-guards.XXXXXX")"
trap 'if [[ $? == 0 ]]; then rm -rf -- "$work_dir"; else echo "Publication guard fixtures retained: $work_dir" >&2; fi' EXIT

fail() {
  echo "error: $*" >&2
  exit 1
}

publish_case="${work_dir}/publish"
mkdir -p "${publish_case}/bin" "${publish_case}/state"
cat > "${publish_case}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == locate-project ]]; then exec "$REAL_CARGO" "$@"; fi
printf 'cargo %s\n' "$*" >> "${TRACE_FILE}"
case "${1:-}" in
  info)
    package="${2%@*}"
    if [[ "${package}" == "ic-query" && -n "${HIDE_LIBRARY_INFO:-}" ]]; then
      exit 1
    fi
    [[ -e "${STATE_DIR}/${package}" ]]
    ;;
  publish)
    package=""
    while [[ "$#" -gt 0 ]]; do
      if [[ "$1" == "-p" ]]; then
        package="${2:-}"
        break
      fi
      shift
    done
    [[ -n "${package}" ]] || exit 2
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
current_version="$(bash "$repo_root/scripts/ci/read-cargo-workspace-version.sh" --stable "$repo_root/Cargo.toml")"
(
  cd "${repo_root}"
  PATH="${publish_case}/bin:${PATH}" TRACE_FILE="${publish_case}/trace" \
    STATE_DIR="${publish_case}/state" CARGO_PUBLISH_INDEX_ATTEMPTS=2 \
    CARGO_PUBLISH_INDEX_DELAY_SECONDS=0 \
    bash scripts/release/publish-workspace.sh
) >/dev/null
printf '%s\n' \
  "observe ic-query@${current_version}" \
  'cargo publish --locked --registry crates-io -p ic-query' \
  "observe ic-query-cli@${current_version}" \
  "cargo info ic-query@${current_version} --registry crates-io" \
  'cargo publish --locked --registry crates-io -p ic-query-cli' \
  > "${publish_case}/expected-trace"
cmp -s "${publish_case}/expected-trace" "${publish_case}/trace" \
  || fail "the workspace publisher ran an unexpected Cargo command sequence"

: > "${publish_case}/trace"
(
  cd "${repo_root}"
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
    cd "$repo_root"
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
    cd "${repo_root}"
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
      cd "$repo_root"
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

make_case="${work_dir}/make-publish"
mkdir -p "${make_case}/bin" "${make_case}/scripts/release" "${make_case}/scripts/ci"
printf '[fixture]\nversion = "9.9.9"\n[workspace.package]\nversion = "%s"\n' \
  "${current_version}" > "${make_case}/Cargo.toml"
cp "${repo_root}/scripts/ci/"{check-crates-io-version.sh,read-cargo-workspace-version.sh} "${make_case}/scripts/ci/"
ln -s "${repo_root}/scripts/release/publish-workspace.sh" \
  "${make_case}/scripts/release/publish-workspace.sh"
cat > "${make_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
head_commit=1111111111111111111111111111111111111111
case "$*" in
  'diff-index --quiet HEAD --') exit "${DIRTY_STATUS:-0}" ;;
  'ls-files --others --exclude-standard')
    [[ -z "${UNTRACKED_PATH:-}" ]] || printf '%s\n' "${UNTRACKED_PATH}"
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
cmp -s "${publish_case}/expected-trace" "${make_case}/trace" \
  || fail "make publish did not delegate once to the retry-safe workspace publisher"

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
  ) >/dev/null 2>&1; then
    fail "make publish accepted a ${invalid_release} release"
  fi
  [[ ! -s "${make_case}/trace" ]] \
    || fail "make publish reached the registry or Cargo for a ${invalid_release} release"
done
