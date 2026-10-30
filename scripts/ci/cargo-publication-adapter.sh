#!/usr/bin/env bash
# Shared companions: scripts/ci/check-make-execution.sh scripts/ci/cargo-publication-selection.pl scripts/ci/run-cargo-publication.sh docs/cargo-publication.md
set -euo pipefail
[[ $# == 1 && ( "$1" == check || "$1" == publish ) ]] || { echo 'select check or publish' >&2; exit 2; }
mode="$1"
self="${BASH_SOURCE[0]%/*}"
bash "$self/check-make-execution.sh"
registry="${CARGO_PUBLISH_REGISTRY:-}"
[[ "$registry" =~ ^[A-Za-z0-9][A-Za-z0-9_-]*$ ]] || { echo 'declare a named publication registry' >&2; exit 2; }
verify_text="${CARGO_PUBLISH_VERIFY_TARGETS:-}"
[[ -n "$verify_text" && "$verify_text" != *$'\n'* && "$verify_text" != *$'\r'* ]] || exit 2
IFS=$' \t' read -r -a verify_targets <<< "$verify_text"
[[ ${#verify_targets[@]} -gt 0 ]] || exit 2
for target in "${verify_targets[@]}"; do
    case "$target" in ''|-*|*=*|publish|publish-check) echo 'publication validation requires non-publication Make targets' >&2; exit 2 ;; esac
done
parent="${CARGO_PUBLISH_EVIDENCE_ROOT:?declare an ignored evidence parent}"
case "$parent" in *$'\n'*|*$'\r'*) exit 2 ;; esac
mkdir -p "$parent"
parent="$(cd -P "$parent" && printf '%s/.' "$PWD")"
parent="${parent%/.}"
selection="$(mktemp -d "$parent/selection.XXXXXX")"
printf 'Publication selection evidence: %s\n' "$selection"
source="$(git rev-parse --verify HEAD)"
printf '%s\n' "$source" > "$selection/source"
status=0
cargo metadata --no-deps --locked --offline --format-version 1 \
    > "$selection/metadata.json" 2> "$selection/metadata.stderr" || status=$?
cat "$selection/metadata.stderr" >&2 || :
[[ "$status" == 0 ]] || exit "$status"
perl "$self/cargo-publication-selection.pl" "$selection/metadata.json" \
    "${CARGO_PUBLISH_PACKAGES:-}" "${CARGO_PUBLISH_SELECTION:-}" "$registry" > "$selection/packages.nul"
arguments=()
while IFS= read -r -d '' package; do arguments+=(--package "$package"); done < "$selection/packages.nul"
[[ ${#arguments[@]} -gt 0 ]] || exit 2
for target in "${verify_targets[@]}"; do
    "${CARGO_PUBLISH_MAKE:-make}" --no-print-directory "$target"
done
# Every invocation uses the same parent lock; unique selection evidence is never
# the runner's lock namespace. Retained incomplete attempts prevent blind replay.
attempt="$parent/$mode.${selection##*/}"
exec bash "$self/run-cargo-publication.sh" "$mode" "$attempt" "$source" "$registry" "${arguments[@]}"
