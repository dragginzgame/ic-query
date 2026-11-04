#!/usr/bin/env bash
set -euo pipefail
needed=true
base="${CI_QUALIFICATION_BASE:-}"
if [[ "${GITHUB_EVENT_NAME:-}" != workflow_dispatch && "$base" =~ ^[0-9a-f]{40,64}$ ]] &&
  git cat-file -e "$base^{commit}" 2>/dev/null; then
  paths="$(git diff --no-ext-diff --no-renames --name-only "$base" HEAD --)"
  needed=false
  while IFS= read -r path; do
    case "$path" in
      ci/release-validation-hosts.json) ;;
      Makefile|make/*|scripts/*|ci/*|rust-toolchain.toml|.shared-tooling.snapshot|.github/workflows/ci.yml)
        needed=true ;;
    esac
  done <<< "$paths"
fi
printf 'needed=%s\n' "$needed"
