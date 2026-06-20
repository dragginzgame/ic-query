#!/usr/bin/env bash
set -euo pipefail

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
if [[ -z "${version}" ]]; then
  echo "error: failed to read package version from Cargo.toml" >&2
  exit 1
fi

minor="${version%.*}"
detail_changelog="docs/changelog/${minor}.md"

if [[ ! -f "${detail_changelog}" ]]; then
  echo "error: missing detailed changelog ${detail_changelog} for version ${version}" >&2
  exit 1
fi

if ! grep -Fq "${version}" CHANGELOG.md; then
  echo "error: CHANGELOG.md does not mention package version ${version}" >&2
  exit 1
fi

if ! grep -Fq "${version}" "${detail_changelog}"; then
  echo "error: ${detail_changelog} does not mention package version ${version}" >&2
  exit 1
fi
