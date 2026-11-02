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

group="${RELEASE_GROUP:-}"
version="$(CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash "$(dirname "${BASH_SOURCE[0]}")/../ci/cargo-release-group.sh" version "$group")" || exit 1
case "$group" in
  query) packages=(ic-query) ;;
  cli) packages=(ic-query-text ic-query-cli) ;;
  *) echo 'select RELEASE_GROUP=query or RELEASE_GROUP=cli' >&2; exit 2 ;;
esac
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
  bash "${version_observer}" "$1" "${2:-$version}" || status=$?
  case "$status" in
    0|1) return "$status" ;;
    *) exit 2 ;;
  esac
}

wait_for_index() {
  local package="$1" selected_version="$2" attempt
  for ((attempt = 1; attempt <= index_attempts; attempt++)); do
    if cargo info "${package}@${selected_version}" --registry "${registry}" >/dev/null 2>&1; then
      echo "${package} ${selected_version} is available from ${registry}"
      return 0
    fi
    if [[ "${attempt}" -eq "${index_attempts}" ]]; then
      echo "error: ${package} ${selected_version} was not visible in ${registry} after ${index_attempts} checks" >&2
      return 1
    fi
    echo "Waiting for ${package} ${selected_version} to appear in ${registry} (${attempt}/${index_attempts})..."
    sleep "${index_delay_seconds}"
  done
}

# The renderer group may advance independently after the initial Query hard cut.
# Require the exact selected Query peer before attempting either presentation crate.
if [[ "$group" == cli ]]; then
  query_version="$(CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 bash "$(dirname "${BASH_SOURCE[0]}")/../ci/cargo-release-group.sh" version query)"
  crate_is_published ic-query "$query_version" || {
    echo "error: publish Query $query_version before the CLI group" >&2; exit 1;
  }
fi
for package in "${packages[@]}"; do
  if crate_is_published "$package"; then
    echo "$package $version is already published; skipping"
  else
    if [[ "$package" == ic-query-text ]]; then wait_for_index ic-query "$query_version"; fi
    if [[ "$package" == ic-query-cli ]]; then wait_for_index ic-query-text "$version"; fi
    publish_package "$package"
  fi
done
