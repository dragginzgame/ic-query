#!/usr/bin/env bash
# Shared companions: scripts/ci/verify-file-checksum.sh
set -euo pipefail
# Caller has already admitted source/recovery and consumed its preparation authority.
[[ $# -ge 2 ]] || exit 2
root="$1" prepare="$2"
shift 2
pins=() checks=() preparation=() digests=()
while [[ $# -gt 0 ]]; do
    [[ $# -ge 2 ]] || exit 2
    case "$1" in
        --pin) pins+=("$2") ;;
        --check) checks+=("$2") ;;
        --prepare) preparation+=("$2") ;;
        *) exit 2 ;;
    esac
    shift 2
done
[[ "$root" == /* && -d "$root" && ${#pins[@]} != 0 && ${#checks[@]} != 0 ]] || exit 2
for target in "${checks[@]}" ${preparation[@]+"${preparation[@]}"}; do
    case "$target" in ''|-*|*=*) exit 2 ;; esac
done
case "$prepare" in 0|1) ;; *) echo 'invalid release cache preparation selection' >&2; exit 2 ;; esac
helper="${BASH_SOURCE[0]%/*}/verify-file-checksum.sh"
for pin in "${pins[@]}"; do digests+=("$(bash "$helper" --print sha256 "$pin")"); done
if [[ "$prepare" == 1 ]]; then
    [[ ${#preparation[@]} != 0 ]] || exit 2
    cargo fetch --locked
    selected=("${preparation[@]}")
else
    cargo fetch --locked --offline
    selected=("${checks[@]}")
fi
for target in "${selected[@]}"; do
    make --no-print-directory -C "$root" "$target"
    for ((index=0; index<${#pins[@]}; index++)); do
        if [[ "$(bash "$helper" --print sha256 "${pins[index]}")" != "${digests[index]}" ]]; then
            printf 'selected tool pins changed during release preflight: %s\n' "${pins[index]}" >&2
            exit 1
        fi
    done
done
