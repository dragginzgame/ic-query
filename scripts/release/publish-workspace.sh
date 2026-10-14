#!/usr/bin/env bash
set -euo pipefail

readonly registry="crates-io"
readonly index_attempts="${CARGO_PUBLISH_INDEX_ATTEMPTS:-12}"
readonly index_delay_seconds="${CARGO_PUBLISH_INDEX_DELAY_SECONDS:-10}"

if ! [[ "${index_attempts}" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: CARGO_PUBLISH_INDEX_ATTEMPTS must be a positive integer" >&2
  exit 2
fi
if ! [[ "${index_delay_seconds}" =~ ^[0-9]+$ ]]; then
  echo "error: CARGO_PUBLISH_INDEX_DELAY_SECONDS must be a non-negative integer" >&2
  exit 2
fi

version="$(bash "$(dirname "${BASH_SOURCE[0]}")/../ci/read-cargo-workspace-version.sh" --stable Cargo.toml)" || exit 1
version_observer="$(dirname "${BASH_SOURCE[0]}")/../ci/check-crates-io-version.sh"
readonly version_observer

source_commit="$(git rev-parse --verify HEAD)"
readonly source_commit
readonly evidence_parent="target/publication"
mkdir -p "$evidence_parent"
readonly workspace_lock="$evidence_parent/.workspace-publication.lock"
mkdir "$workspace_lock"
# shellcheck disable=SC2329 # Registered as the EXIT trap below.
finish() {
  local status=$?
  if [[ -e "$evidence_parent/.cargo-publication.lock" ]]; then
    printf 'Publication interrupted; inspect evidence and both locks in %s\n' "$evidence_parent" >&2 || :
    [[ "$status" != 0 ]] || status=1
  else
    if ! { rm -f -- "$workspace_lock/owner" && rmdir -- "$workspace_lock"; }; then
      [[ "$status" != 0 ]] || status=1
    fi
  fi
  exit "$status"
}
trap finish EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
printf '%s\n' "$$" > "$workspace_lock/owner"

publish_package() {
  local attempt
  attempt="$(mktemp -d "$evidence_parent/$1.XXXXXX")"
  rmdir -- "$attempt"
  bash "$(dirname "${BASH_SOURCE[0]}")/../ci/run-cargo-publication.sh" \
    publish "$attempt" "$source_commit" "$registry" --package "$1@$version"
}

crate_is_published() {
  local status=0
  bash "${version_observer}" "$1" "${version}" || status=$?
  case "$status" in
    0|1) return "$status" ;;
    *) exit 2 ;;
  esac
}

wait_for_library_index() {
  local attempt
  for ((attempt = 1; attempt <= index_attempts; attempt++)); do
    if cargo info "ic-query@${version}" --registry "${registry}" >/dev/null 2>&1; then
      echo "ic-query ${version} is available from ${registry}"
      return 0
    fi
    if [[ "${attempt}" -eq "${index_attempts}" ]]; then
      echo "error: ic-query ${version} was not visible in ${registry} after ${index_attempts} checks" >&2
      return 1
    fi
    echo "Waiting for ic-query ${version} to appear in ${registry} (${attempt}/${index_attempts})..."
    sleep "${index_delay_seconds}"
  done
}

if crate_is_published ic-query; then
  echo "ic-query ${version} is already published; skipping"
else
  publish_package ic-query
fi
if crate_is_published ic-query-cli; then
  echo "ic-query-cli ${version} is already published; skipping"
else
  wait_for_library_index
  publish_package ic-query-cli
fi
