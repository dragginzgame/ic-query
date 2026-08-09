#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -gt 1 ]]; then
  echo "Usage: $0 [VERSION]" >&2
  exit 2
fi

version="${1:-}"
if [[ -z "${version}" ]]; then
  version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
fi
if [[ -z "${version}" ]]; then
  echo "error: failed to read package version from Cargo.toml" >&2
  exit 1
fi

if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: unsupported version format ${version}" >&2
  exit 2
fi

minor="${version%.*}"
detail_changelog="docs/changelog/${minor}.md"
root_version_marker="- \`${version}\`"
version_pattern="${version//./\\.}"

if [[ ! -f "${detail_changelog}" ]]; then
  echo "error: missing detailed changelog ${detail_changelog} for version ${version}" >&2
  exit 1
fi

if ! head_detail_changelog="$(git show "HEAD:${detail_changelog}" 2>/dev/null)"; then
  echo "error: detailed changelog ${detail_changelog} is not committed in HEAD" >&2
  exit 1
fi

if ! head_root_changelog="$(git show HEAD:CHANGELOG.md 2>/dev/null)"; then
  echo "error: CHANGELOG.md is not committed in HEAD" >&2
  exit 1
fi

if ! grep -Fq -- "${root_version_marker}" CHANGELOG.md; then
  echo "error: CHANGELOG.md has no release-ledger entry for package version ${version}" >&2
  exit 1
fi

if ! grep -Fq -- "${root_version_marker}" <<<"${head_root_changelog}"; then
  echo "error: CHANGELOG.md in HEAD has no release-ledger entry for package version ${version}" >&2
  exit 1
fi

if ! grep -Eq -- "^## ${version_pattern}( - .+)?$" "${detail_changelog}"; then
  echo "error: ${detail_changelog} has no heading for package version ${version}" >&2
  exit 1
fi

if ! grep -Eq -- "^## ${version_pattern}( - .+)?$" <<<"${head_detail_changelog}"; then
  echo "error: ${detail_changelog} in HEAD has no heading for package version ${version}" >&2
  exit 1
fi
