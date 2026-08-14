#!/usr/bin/env bash
set -euo pipefail

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
if [[ -z "${version}" ]]; then
  echo "error: failed to read package version from Cargo.toml" >&2
  exit 1
fi

if git rev-parse "v${version}" >/dev/null 2>&1; then
  echo "error: tag v${version} already exists; aborting" >&2
  exit 1
fi

if ! git diff --quiet --; then
  echo "error: unstaged release changes remain; run make release-stage and review them" >&2
  exit 1
fi

untracked_paths="$(git ls-files --others --exclude-standard)"
if [[ -n "${untracked_paths}" ]]; then
  echo "error: untracked files remain; release-commit will not create a partial release" >&2
  printf '%s\n' "${untracked_paths}" >&2
  exit 1
fi

if git diff --cached --quiet --; then
  echo "error: no release changes are staged" >&2
  exit 1
fi

if ! staged_paths="$(git diff --cached --name-only --diff-filter=ACDMRTUXB --)"; then
  echo "error: failed to inspect staged release paths" >&2
  exit 1
fi

unexpected_staged=0
while IFS= read -r staged_path; do
  case "${staged_path}" in
    Cargo.lock | Cargo.toml | README.md | \
      crates/ic-query/Cargo.toml | crates/ic-query-cli/Cargo.toml | \
      docs/library-usage.md)
      ;;
    *)
      echo "error: unexpected staged release path: ${staged_path}" >&2
      unexpected_staged=1
      ;;
  esac
done <<< "${staged_paths}"
if [[ "${unexpected_staged}" -ne 0 ]]; then
  exit 1
fi

git commit -m "Release ${version}"

# A commit hook may modify files after the pre-commit check. Never attach the
# release tag unless the resulting commit is the complete clean release state.
post_commit_untracked="$(git ls-files --others --exclude-standard)"
if ! git diff-index --quiet HEAD -- \
  || [[ -n "${post_commit_untracked}" ]]; then
  echo "error: release commit left working-tree changes; tag v${version} was not created" >&2
  exit 1
fi

git tag -a "v${version}" -m "Release ${version}"
