#!/usr/bin/env bash
set -euo pipefail

failed=0

while IFS= read -r -d '' workflow; do
  line_number=0
  while IFS= read -r line || [[ -n "${line}" ]]; do
    line_number=$((line_number + 1))
    [[ "${line}" =~ ^[[:space:]]*# ]] && continue
    if [[ "${line}" =~ uses:[[:space:]]*([^[:space:]#]+) ]]; then
      action="${BASH_REMATCH[1]}"
      case "${action}" in
        ./* | docker://*) continue ;;
      esac
      if [[ "${action}" != *@* ]]; then
        echo "error: ${workflow}:${line_number}: action is missing an explicit ref: ${action}" >&2
        failed=1
        continue
      fi
      ref="${action##*@}"
      if [[ ! "${ref}" =~ ^[0-9a-fA-F]{40}$ ]]; then
        echo "error: ${workflow}:${line_number}: action must be pinned to a full commit SHA: ${action}" >&2
        failed=1
      fi
    fi
  done < "${workflow}"
done < <(find .github/workflows -type f \( -name '*.yml' -o -name '*.yaml' \) -print0)

exit "${failed}"
