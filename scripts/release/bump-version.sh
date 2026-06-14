#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo "Usage: $0 patch|minor|major" >&2
}

bump="${1:-patch}"
case "${bump}" in
    patch | minor | major) ;;
    *)
        usage
        exit 2
        ;;
esac

if ! cargo set-version --help >/dev/null 2>&1; then
    echo "error: cargo set-version is required; install cargo-edit first" >&2
    echo "hint: cargo install cargo-edit" >&2
    exit 1
fi

previous_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
if [[ -z "${previous_version}" ]]; then
    echo "error: failed to read package version from Cargo.toml" >&2
    exit 1
fi

cargo set-version --bump "${bump}" >/dev/null

new_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
if [[ "${previous_version}" == "${new_version}" ]]; then
    echo "Version unchanged (${new_version})"
    exit 0
fi

if [[ -f Cargo.lock ]]; then
    cargo generate-lockfile >/dev/null
fi

if git rev-parse "v${new_version}" >/dev/null 2>&1; then
    echo "error: tag v${new_version} already exists; aborting" >&2
    exit 1
fi

echo "Bumped: ${previous_version} -> ${new_version}"
echo "Next:"
echo "  git diff"
echo "  make release-stage"
echo "  make release-commit"
echo "  make release-push"
