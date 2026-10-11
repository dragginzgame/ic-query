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
if [[ -d "$repository/.tools" ]]; then
  # Preserve local evidence even when the rejected store caused setup to fail.
  bash "$root/scripts/ci/archive-evidence.sh" "$evidence_dir/local-tools.tar.gz" "$repository" .tools
fi
inputs=()
if bash "$root/scripts/ci/select-tool-evidence.sh" full "$repository" "$evidence_dir" \
  > "$evidence_dir/selections.nul" 2> "$evidence_dir/selection.log"; then
  cat "$evidence_dir/selection.log" >&2
else
  status=$?
  cat "$evidence_dir/selection.log" >&2
  exit "$status"
fi
while IFS= read -r -d '' selected_root && IFS= read -r -d '' selected_path; do
  # Local references and retained bundles are already in the physical tree archive.
  [[ "$selected_root" != "$repository" ]] || continue
  inputs+=("$selected_root" "$selected_path")
done < "$evidence_dir/selections.nul"
if [[ ${#inputs[@]} != 0 ]]; then
  bash "$root/scripts/ci/archive-evidence.sh" "$evidence_dir/shared-tools.tar.gz" "${inputs[@]}"
fi
