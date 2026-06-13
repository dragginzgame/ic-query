#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo "Usage: $0 patch|minor|major" >&2
}

bump="${1:-}"
case "${bump}" in
    patch | minor | major) ;;
    *)
        usage
        exit 2
        ;;
esac

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "error: release bump must run inside a git worktree" >&2
    exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
    echo "error: release bump requires a clean worktree" >&2
    echo "" >&2
    git status --short >&2
    exit 1
fi

if ! cargo set-version --help >/dev/null 2>&1; then
    echo "error: cargo set-version is required; install cargo-edit first" >&2
    echo "hint: cargo install cargo-edit" >&2
    exit 1
fi

mutated=0
committed=0
cleanup_failed_release() {
    status=$?
    if [[ "${status}" -ne 0 && "${mutated}" -eq 1 && "${committed}" -eq 0 ]]; then
        echo "error: release failed after version mutation; restoring Cargo.toml and Cargo.lock" >&2
        git restore -- Cargo.toml Cargo.lock || true
    fi
}
trap cleanup_failed_release EXIT

make test

if [[ -n "$(git status --porcelain)" ]]; then
    echo "error: worktree changed while running make test" >&2
    echo "" >&2
    git status --short >&2
    exit 1
fi

current_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
if [[ ! "${current_version}" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
    echo "error: unsupported package version '${current_version}'" >&2
    exit 1
fi

major="${BASH_REMATCH[1]}"
minor="${BASH_REMATCH[2]}"
patch="${BASH_REMATCH[3]}"

major="$((10#${major}))"
minor="$((10#${minor}))"
patch="$((10#${patch}))"

case "${bump}" in
    patch)
        patch="$((patch + 1))"
        ;;
    minor)
        minor="$((minor + 1))"
        patch="0"
        ;;
    major)
        major="$((major + 1))"
        minor="0"
        patch="0"
        ;;
esac

new_version="${major}.${minor}.${patch}"
tag="v${new_version}"

if git rev-parse --verify --quiet "refs/tags/${tag}" >/dev/null; then
    echo "error: tag ${tag} already exists" >&2
    exit 1
fi

cargo set-version "${new_version}"
mutated=1
cargo generate-lockfile --offline
cargo package --locked --allow-dirty

git add Cargo.toml Cargo.lock
git commit -m "chore: release ${tag}"
committed=1
git tag -a "${tag}" -m "${tag}"

echo "Released ${tag}"
echo "Push with: git push origin HEAD ${tag}"
