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

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: failed to read a release version from Cargo.toml" >&2
  exit 1
fi

crate_is_available() {
  local package="$1"
  cargo info "${package}@${version}" --registry "${registry}" >/dev/null 2>&1
}

wait_for_library_index() {
  local attempt
  for ((attempt = 1; attempt <= index_attempts; attempt++)); do
    if crate_is_available ic-query; then
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

if crate_is_available ic-query; then
  echo "ic-query ${version} is already published; skipping"
else
  cargo publish --locked --registry "${registry}" -p ic-query
  wait_for_library_index
fi

if crate_is_available ic-query-cli; then
  echo "ic-query-cli ${version} is already published; skipping"
else
  cargo publish --locked --registry "${registry}" -p ic-query-cli
fi
