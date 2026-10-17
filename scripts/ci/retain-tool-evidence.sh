#!/usr/bin/env bash
set -euo pipefail

# Consumer retention layout; Shared Tooling owns bundle admission and archiving.
[[ $# == 2 ]] || { echo 'usage: retain-tool-evidence.sh REPOSITORY TEMP-ROOT' >&2; exit 2; }
repository="$1"
temp_root="$2"
[[ "$repository" == /* && "$temp_root" == /* && -d "$repository" && -d "$temp_root" ]] || exit 1
[[ ! -L "$repository/.tools" ]] || exit 0
root="$(cd -P "${BASH_SOURCE[0]%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
evidence_dir="$(mktemp -d "$temp_root/ic-query-tool-evidence.XXXXXX")"
mkdir "$evidence_dir/diagnostics"
bash "$root/scripts/ci/collect-tool-evidence.sh" "$evidence_dir/tools.tar.gz" \
  "$repository" "$evidence_dir/diagnostics" full \
  "$root/ci/tool-versions.env" "${IC_TOOL_PINS:-$root/ci/ic-tools-query.tsv}"
