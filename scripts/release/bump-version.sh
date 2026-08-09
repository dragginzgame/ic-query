#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 patch|minor|major" >&2
}

cleanup_release_build_artifacts() {
  echo "Cleaning Cargo build artifacts after the successful release gate..."
  if ! cargo clean; then
    echo "warning: cargo clean failed after the successful release gate" >&2
  fi
}

update_documented_dependency_versions() {
  local release_line="${1%.*}"
  local usage_doc

  for usage_doc in README.md docs/library-usage.md; do
    [[ -f "${usage_doc}" ]] || continue
    IC_QUERY_RELEASE_LINE="${release_line}" perl -0pi -e '
      s/(^ic-query = \{ version = ")[0-9]+\.[0-9]+(".*$)/$1$ENV{IC_QUERY_RELEASE_LINE}$2/mg
    ' "${usage_doc}"
  done
}

bump="${1:-patch}"
case "${bump}" in
  patch | minor | major) ;;
  *)
    usage
    exit 2
    ;;
esac

previous_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
if [[ -z "${previous_version}" ]]; then
  echo "error: failed to read package version from Cargo.toml" >&2
  exit 1
fi

IFS=. read -r major minor patch_extra <<< "${previous_version}"
patch="${patch_extra%%[-+]*}"
if [[ ! "${major}" =~ ^[0-9]+$ || ! "${minor}" =~ ^[0-9]+$ || ! "${patch}" =~ ^[0-9]+$ ]]; then
  echo "error: unsupported version format ${previous_version}" >&2
  exit 1
fi

case "${bump}" in
  patch)
    patch=$((patch + 1))
    ;;
  minor)
    minor=$((minor + 1))
    patch=0
    ;;
  major)
    major=$((major + 1))
    minor=0
    patch=0
    ;;
esac

new_version="${major}.${minor}.${patch}"

if git rev-parse "v${new_version}" >/dev/null 2>&1; then
  echo "error: tag v${new_version} already exists; aborting" >&2
  exit 1
fi

echo "Checking committed changelog for target version ${new_version}..."
bash scripts/ci/check-changelog-version.sh "${new_version}"

echo "Running full CI gate before version bump..."
make --no-print-directory ensure-clean
CHANGELOG_VERSION="${new_version}" make --no-print-directory ci

perl -0pi -e "s/version = \"\\Q${previous_version}\\E\"/version = \"${new_version}\"/g" Cargo.toml
update_documented_dependency_versions "${new_version}"

if [[ -f Cargo.lock ]]; then
    cargo generate-lockfile >/dev/null
fi

cleanup_release_build_artifacts

echo "Bumped: ${previous_version} -> ${new_version}"
echo "Next:"
echo "  git diff"
echo "  make release-stage"
echo "  make release-commit"
echo "  make release-push"
