#!/usr/bin/env bash
set -euo pipefail

package_retries="${CARGO_PACKAGE_RETRIES:-3}"

if ! [[ "${package_retries}" =~ ^[1-9][0-9]*$ ]]; then
  echo "error: CARGO_PACKAGE_RETRIES must be a positive integer" >&2
  exit 2
fi

attempt=1
while true; do
  echo "cargo package attempt ${attempt}/${package_retries}: CARGO_HTTP_MULTIPLEXING=${CARGO_HTTP_MULTIPLEXING:-unset} CARGO_NET_RETRY=${CARGO_NET_RETRY:-unset} cargo package $*"
  if cargo package "$@"; then
    exit 0
  fi

  status="$?"
  if [[ "${attempt}" -ge "${package_retries}" ]]; then
    exit "${status}"
  fi

  sleep_seconds=$((attempt * 5))
  echo "warning: cargo package failed with status ${status}; retrying in ${sleep_seconds}s" >&2
  sleep "${sleep_seconds}"
  attempt=$((attempt + 1))
done
