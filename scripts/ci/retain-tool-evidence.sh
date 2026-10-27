#!/usr/bin/env bash
set -euo pipefail

# Consumer retention layout; Shared Tooling owns bundle admission and archiving.
[[ $# == 2 ]] || { echo 'usage: retain-tool-evidence.sh REPOSITORY TEMP-ROOT' >&2; exit 2; }
repository="$1"
temp_root="$2"
[[ "$repository" == /* && "$temp_root" == /* && -d "$repository" && -d "$temp_root" ]] || exit 1
root="$(cd -P "${BASH_SOURCE[0]%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
evidence_dir="$(mktemp -d "$temp_root/ic-query-tool-evidence.XXXXXX")"
mkdir "$evidence_dir/diagnostics"
# Raw fixtures intentionally contain names that upload-artifact rejects. Keep
# their exact names and bytes inside a safe archive instead of uploading the tree.
validation_inputs=()
if [[ -e "$temp_root/ic-query-validation" || -L "$temp_root/ic-query-validation" ]]; then
  validation_inputs+=("$temp_root" ic-query-validation)
fi
for formatting in "$temp_root"/formatting.*; do
  [[ -e "$formatting" || -L "$formatting" ]] || continue
  validation_inputs+=("$temp_root" "${formatting##*/}")
done
if [[ -e "$repository/target/validation-failures" || -L "$repository/target/validation-failures" ]]; then
  validation_inputs+=("$repository" target/validation-failures)
fi
status=0
if [[ "${#validation_inputs[@]}" != 0 ]]; then
  bash "$root/scripts/ci/archive-evidence.sh" "$evidence_dir/validation.tar.gz" \
    "${validation_inputs[@]}" || status=$?
fi
[[ ! -L "$repository/.tools" ]] || exit "$status"
# Query selects Testkit's bundle directory; Testkit owns the retained attempt
# layout within it. Preserve partial downloads and stderr without parsing them.
inputs=()
if [[ -e "$repository/.tools/testkit-server" || -L "$repository/.tools/testkit-server" ]]; then
  inputs+=("$repository" .tools/testkit-server)
fi
bash "$root/scripts/ci/collect-tool-evidence.sh" "$evidence_dir/tools.tar.gz" \
  "$repository" "$evidence_dir/diagnostics" full \
  "$root/ci/tool-versions.env" "${IC_TOOL_PINS:-$root/ci/ic-tools.tsv}" \
  ${inputs[@]+"${inputs[@]}"} || {
    tool_status=$?
    [[ "$status" != 0 ]] || status="$tool_status"
  }
exit "$status"
