#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if [[ "$#" -gt 1 ]]; then
  echo "Usage: $0 [SOURCE_ROOT]" >&2
  exit 2
fi

scan_root="${1:-${repo_root}/crates}"
if [[ ! -d "${scan_root}" ]]; then
  echo "error: schema-version source root does not exist: ${scan_root}" >&2
  exit 2
fi

readonly nonconforming_schema_constant='(?:pub(?:\([^)]*\))?\s+)?const\s+[A-Z][A-Z0-9_]*SCHEMA_VERSION[A-Z0-9_]*\s*:\s*(?:u8|u16|u32|u64|usize)\s*=\s*(?:0|[2-9][0-9]*)\s*;'

if matches="$(rg --line-number --glob '*.rs' "${nonconforming_schema_constant}" "${scan_root}")"; then
  echo "error: pre-1.0 schema constants must remain 1" >&2
  printf '%s\n' "${matches}" >&2
  exit 1
else
  status="$?"
  if [[ "${status}" -ne 1 ]]; then
    exit "${status}"
  fi
fi
