#!/usr/bin/env bash
set -euo pipefail

failed=0
packages=(ic-query ic-query-cli)

for package in "${packages[@]}"; do
  if package_files="$(cargo package -p "${package}" --list --allow-dirty)"; then
    :
  else
    status="$?"
    echo "error: failed to list ${package} package contents" >&2
    exit "${status}"
  fi
  while IFS= read -r package_file; do
    case "${package_file}" in
      .github/* | .gitignore | AGENTS.md | docs/governance/* | rust-toolchain.toml | scripts/dev/*)
        echo "error: internal file is included in ${package} package: ${package_file}" >&2
        failed=1
        ;;
    esac
  done <<<"${package_files}"
done

exit "${failed}"
