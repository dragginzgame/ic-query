#!/usr/bin/env bash
set -euo pipefail

failed=0

while IFS= read -r package_file; do
  case "${package_file}" in
    .github/* | .gitignore | AGENTS.md | docs/governance/* | rust-toolchain.toml | scripts/dev/*)
      echo "error: internal file is included in crate package: ${package_file}" >&2
      failed=1
      ;;
  esac
done < <(cargo package --list --allow-dirty)

exit "${failed}"
