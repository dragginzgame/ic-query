#!/usr/bin/env bash
# Shared companions: scripts/ci/collect-tool-evidence.sh scripts/ci/fixture-lifecycle.sh
set -euo pipefail
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/collect-tool-evidence.XXXXXX")"
source "$ROOT/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Collector fixture retained"' EXIT
mkdir -p "$fixture/bin" "$fixture/repository/.tools/rust/build" \
    "$fixture/repository/.tools/host-set.failed" "$fixture/outside" "$fixture/logs"
printf 'local build\n' > "$fixture/repository/.tools/rust/build/log"
printf 'failed candidate\n' > "$fixture/repository/.tools/host-set.failed/log"
printf 'private\n' > "$fixture/outside/private"
printf 'original failure\n' > "$fixture/logs/log"
ln -s "$fixture/outside" "$fixture/repository/.tools/ic-set.reference"
ln -s ic-set.reference "$fixture/repository/.tools/ic"
printf '#!%s\n' "$BASH" > "$fixture/bin/bash"
cat >> "$fixture/bin/bash" <<'STUB'
set -euo pipefail
case "${1:-}" in
    */select-tool-evidence.sh)
        if [[ "$COLLECTOR_CASE" == complete ]]; then
            printf '%s\0%s\0' "$COLLECTOR_REPOSITORY" .tools/rust
        else
            printf '%s\0private\0truncated\0' "$COLLECTOR_OUTSIDE"
            echo 'original selector failure' >&2
            exit 23
        fi ;;
    *) exec "$COLLECTOR_REAL_BASH" "$@" ;;
esac
STUB
chmod +x "$fixture/bin/bash"
export COLLECTOR_REAL_BASH="$BASH" COLLECTOR_REPOSITORY="$fixture/repository" COLLECTOR_OUTSIDE="$fixture/outside"
for mode in complete partial redirected; do
    mkdir "$fixture/$mode" "$fixture/$mode/unpacked"
    if [[ "$mode" == redirected ]]; then
        mv "$fixture/repository/.tools" "$fixture/retained-tools"
        ln -s "$fixture/retained-tools" "$fixture/repository/.tools"
    fi
    status=0
    COLLECTOR_CASE="$mode" PATH="$fixture/bin:$PATH" "$BASH" \
        "$ROOT/scripts/ci/collect-tool-evidence.sh" "$fixture/$mode.tar.gz" \
        "$fixture/repository" "$fixture/$mode" compact unused-host unused-ic \
        "$fixture/logs" log > "$fixture/$mode/output" 2> "$fixture/$mode/error" || status=$?
    expected=1; [[ "$mode" != complete ]] || expected=0
    [[ "$status" == "$expected" ]] || exit 1
    [[ "$(cat "$fixture/$mode/output")" == "$fixture/$mode.tar.gz" ]] || exit 1
    tar -xzf "$fixture/$mode.tar.gz" -C "$fixture/$mode/unpacked"
    cmp "$fixture/logs/log" "$fixture/$mode/unpacked/log"
    [[ ! -e "$fixture/$mode/unpacked/private" ]] || exit 1
    if [[ "$mode" != redirected ]]; then
        cmp "$fixture/repository/.tools/rust/build/log" "$fixture/$mode/unpacked/.tools/rust/build/log"
    else
        [[ ! -e "$fixture/$mode/unpacked/.tools" ]] || exit 1
    fi
    if [[ "$mode" != complete ]]; then
        [[ "$(cat "$fixture/$mode/unpacked/$mode/selection-status")" == 23 ]] || exit 1
        grep -Fq 'original selector failure' "$fixture/$mode/unpacked/$mode/selection.log"
        printf '%s\0private\0truncated\0' "$fixture/outside" > "$fixture/expected-partial"
        cmp "$fixture/expected-partial" "$fixture/$mode/unpacked/$mode/selections.nul"
        if [[ "$mode" == partial ]]; then
            cmp "$fixture/repository/.tools/host-set.failed/log" "$fixture/$mode/unpacked/.tools/host-set.failed/log"
            [[ -L "$fixture/$mode/unpacked/.tools/ic-set.reference" ]] || exit 1
        fi
    fi
done
# Archive failure must not be confused with safely archived partial evidence.
mkdir "$fixture/archive-failure"
status=0
COLLECTOR_CASE=complete PATH="$fixture/bin:$PATH" "$BASH" \
    "$ROOT/scripts/ci/collect-tool-evidence.sh" "$fixture/partial.tar.gz" \
    "$fixture/repository" "$fixture/archive-failure" full unused-host unused-ic \
    "$fixture/logs" log > "$fixture/archive-failure/output" 2>&1 || status=$?
[[ "$status" == 2 && "$(cat "$fixture/archive-failure/archive-status")" == 1 ]] || exit 1
fixture_complete=true
echo 'Collector success, failed partial selection, redirected roots and archive failure passed'
