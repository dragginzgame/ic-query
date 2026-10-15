#!/usr/bin/env bash
# Shared companions: scripts/ci/cargo-release-preflight.sh scripts/ci/fixture-lifecycle.sh
set -euo pipefail
ROOT="$(cd "${BASH_SOURCE[0]%/*}/../.." && pwd -P)"
source "$ROOT/scripts/ci/fixture-lifecycle.sh"
fixture="$(fixture_create "${TMPDIR:-/tmp}/cargo-preflight.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Cargo preflight fixture retained"' EXIT
mkdir -p "$fixture/bin" "$fixture/tool root" "$fixture/selected source"
printf 'first pin\n' > "$fixture/first pins"
printf 'second pin\n' > "$fixture/second pins"
cat > "$fixture/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
printf 'cargo:%s:%s\n' "$PWD" "$*" >> "$PREFLIGHT_EVENTS"
[[ -z "${IC_METRICS_RELEASE_CACHE_PREPARE+x}" ]] || exit 99
exit "${PREFLIGHT_FETCH_STATUS:-0}"
CARGO
cat > "$fixture/bin/make" <<'MAKE'
#!/usr/bin/env bash
[[ $# == 4 && "$1" == --no-print-directory && "$2" == -C && "$3" == "$PREFLIGHT_TOOL_ROOT" ]] || exit 99
printf 'make:%s\n' "$4" >> "$PREFLIGHT_EVENTS"
if [[ "${PREFLIGHT_CHANGE_PIN:-no}" == yes ]]; then printf changed >> "$PREFLIGHT_SECOND_PIN"; fi
exit "${PREFLIGHT_MAKE_STATUS:-0}"
MAKE
chmod +x "$fixture/bin/cargo" "$fixture/bin/make"
export PATH="$fixture/bin:$PATH" PREFLIGHT_EVENTS="$fixture/events" PREFLIGHT_TOOL_ROOT="$fixture/tool root" PREFLIGHT_SECOND_PIN="$fixture/second pins"
cd "$fixture/selected source"
run() {
    bash "$ROOT/scripts/ci/cargo-release-preflight.sh" "$PREFLIGHT_TOOL_ROOT" "$1" \
        --pin "$fixture/first pins" --pin "$fixture/second pins" --check offline-check \
        --prepare install --prepare offline-check
}
for mode in 0 1; do
    : > "$PREFLIGHT_EVENTS"
    run "$mode"
    if [[ "$mode" == 0 ]]; then
        printf 'cargo:%s:fetch --locked --offline\nmake:offline-check\n' "$PWD" > "$fixture/expected"
    else
        printf 'cargo:%s:fetch --locked\nmake:install\nmake:offline-check\n' "$PWD" > "$fixture/expected"
    fi
    cmp "$fixture/expected" "$PREFLIGHT_EVENTS"
done
for kind in fetch make pin invalid; do
    : > "$PREFLIGHT_EVENTS"
    status=0
    case "$kind" in
        fetch) PREFLIGHT_FETCH_STATUS=43 run 1 > "$fixture/$kind.log" 2>&1 || status=$?; expected=43 ;;
        make) PREFLIGHT_MAKE_STATUS=49 run 1 > "$fixture/$kind.log" 2>&1 || status=$?; expected=49 ;;
        pin) PREFLIGHT_CHANGE_PIN=yes run 1 > "$fixture/$kind.log" 2>&1 || status=$?; expected=1 ;;
        invalid) run 2 > "$fixture/$kind.log" 2>&1 || status=$?; expected=2 ;;
    esac
    [[ "$status" == "$expected" ]] || exit 1
    case "$kind" in
        fetch) [[ $(wc -l < "$PREFLIGHT_EVENTS") -eq 1 ]] || exit 1 ;;
        invalid) [[ ! -s "$PREFLIGHT_EVENTS" ]] || exit 1 ;;
        *) [[ $(wc -l < "$PREFLIGHT_EVENTS") -eq 2 ]] || exit 1 ;;
    esac
done
fixture_complete=true
echo 'Locked Cargo preparation, ordered tool checks and multi-pin refusal passed'
