#!/usr/bin/env bash
# Shared companions: scripts/ci/cargo-release-preflight.sh scripts/ci/run-release.sh scripts/ci/fixture-lifecycle.sh
set -euo pipefail
unset RELEASE_MAKE PREFLIGHT_REAL_FETCH
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES RELEASE_COMMIT
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES
export RELEASE_DELIVERY=direct
ROOT="$(cd "${BASH_SOURCE[0]%/*}/../.." && pwd -P)"
source "$ROOT/scripts/ci/fixture-lifecycle.sh"
fixture="$(fixture_create "${TMPDIR:-/tmp}/cargo-preflight.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Cargo preflight fixture retained"' EXIT
mkdir -p "$fixture/bin" "$fixture/tool root" "$fixture/selected source"
printf 'first pin\n' > "$fixture/first pins"
printf 'second pin\n' > "$fixture/second pins"
PREFLIGHT_REAL_MAKE="$(command -v make)"
export PREFLIGHT_REAL_MAKE
PREFLIGHT_REAL_CARGO="$(command -v cargo)"
export PREFLIGHT_REAL_CARGO
cat > "$fixture/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo:%s:%s\n' "$PWD" "$*" >> "$PREFLIGHT_EVENTS"
[[ -z "${IC_METRICS_RELEASE_CACHE_PREPARE+x}" ]] || exit 99
if [[ "${PREFLIGHT_REAL_FETCH:-0}" == 1 ]]; then exec "$PREFLIGHT_REAL_CARGO" "$@"; fi
[[ "${PREFLIGHT_FETCH_STATUS:-0}" == 0 ]] || exit "$PREFLIGHT_FETCH_STATUS"
if [[ -n "${PREFLIGHT_CACHE:-}" && ! -e "$PREFLIGHT_CACHE/input" ]]; then
    if [[ "$*" == *' --offline' || "${CARGO_NET_OFFLINE:-false}" == true ||
          "${PREFLIGHT_CONFIG_OFFLINE:-false}" == true ]]; then exit 43; fi
    printf 'prepared locked input\n' > "$PREFLIGHT_CACHE/input"
fi
CARGO
cat > "$fixture/bin/make" <<'MAKE'
#!/usr/bin/env bash
set -euo pipefail
[[ $# == 4 && "$1" == --no-print-directory && "$2" == -C && "$3" == "$PREFLIGHT_TOOL_ROOT" ]] || exit 99
printf 'make:%s\n' "$4" >> "$PREFLIGHT_EVENTS"
if [[ "${PREFLIGHT_CHANGE_PIN:-no}" == yes ]]; then printf changed >> "$PREFLIGHT_SECOND_PIN"; fi
[[ "${PREFLIGHT_MAKE_STATUS:-0}" == 0 ]] || exit "$PREFLIGHT_MAKE_STATUS"
[[ "${PREFLIGHT_FAIL_TARGET:-}" != "$4" ]] || exit 49
if [[ -n "${PREFLIGHT_CACHE:-}" ]]; then
    case "$4" in
        install)
            if [[ ! -e "$PREFLIGHT_CACHE/tool" && "${CARGO_NET_OFFLINE:-false}" == true ]]; then exit 57; fi
            printf 'prepared executable\n' > "$PREFLIGHT_CACHE/tool" ;;
        offline-check) [[ -e "$PREFLIGHT_CACHE/tool" ]] || exit 53 ;;
        *) exit 99 ;;
    esac
fi
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
# Release adapters may select one Make executable, including a path with spaces.
# Its failure must stop before a later target without falling back to ambient Make.
sed 's/make:%s/selected-make:%s/' "$fixture/bin/make" > "$fixture/selected make"
chmod +x "$fixture/selected make"
: > "$PREFLIGHT_EVENTS"
RELEASE_MAKE="$fixture/selected make" run 1
printf 'cargo:%s:fetch --locked\nselected-make:install\nselected-make:offline-check\n' "$PWD" > "$fixture/expected"
cmp "$fixture/expected" "$PREFLIGHT_EVENTS"
: > "$PREFLIGHT_EVENTS"
status=0
RELEASE_MAKE="$fixture/selected make" PREFLIGHT_MAKE_STATUS=39 run 1 \
    > "$fixture/selected-failure.log" 2>&1 || status=$?
[[ "$status" == 39 ]] || exit 1
printf 'cargo:%s:fetch --locked\nselected-make:install\n' "$PWD" > "$fixture/expected"
cmp "$fixture/expected" "$PREFLIGHT_EVENTS"
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
# Shared owns Cargo-policy admission and simulated cache/tool states. Consumers
# retain preparation-authority routing, selected pins and Git custody.
printf 'unchanged manifest\n' > Cargo.toml
printf 'unchanged lock\n' > Cargo.lock
cp Cargo.toml "$fixture/original.toml"
cp Cargo.lock "$fixture/original.lock"
# Scenario exports are intentionally private to each independent subshell.
# shellcheck disable=SC2030
for scenario in cold standalone-offline environment-offline config-offline fetch-failure warm-offline \
    tool-cold tool-install-failure tool-check-failure tool-standalone-missing tool-offline-missing; do (
    export PREFLIGHT_CACHE="$fixture/$scenario"
    mkdir "$PREFLIGHT_CACHE"
    printf 'old executable\n' > "$PREFLIGHT_CACHE/old-tool"
    printf 'prepared executable\n' > "$PREFLIGHT_CACHE/tool"
    printf 'first pin\n' > "$fixture/first pins"
    printf 'second pin\n' > "$fixture/second pins"
    : > "$PREFLIGHT_EVENTS"
    unset CARGO_NET_OFFLINE PREFLIGHT_CONFIG_OFFLINE PREFLIGHT_FETCH_STATUS PREFLIGHT_FAIL_TARGET
    mode=1 expected=0
    case "$scenario" in
        standalone-offline) mode=0; expected=43 ;;
        environment-offline) export CARGO_NET_OFFLINE=true; expected=43 ;;
        config-offline) export PREFLIGHT_CONFIG_OFFLINE=true; expected=43 ;;
        fetch-failure) export PREFLIGHT_FETCH_STATUS=47; expected=47 ;;
        warm-offline) export CARGO_NET_OFFLINE=true; printf 'prepared locked input\n' > "$PREFLIGHT_CACHE/input" ;;
        tool-*) printf 'prepared locked input\n' > "$PREFLIGHT_CACHE/input" ;;
    esac
    case "$scenario" in
        tool-cold) rm "$PREFLIGHT_CACHE/tool" ;;
        tool-install-failure) export PREFLIGHT_FAIL_TARGET=install; expected=49 ;;
        tool-check-failure) export PREFLIGHT_FAIL_TARGET=offline-check; expected=49 ;;
        tool-standalone-missing) mode=0; rm "$PREFLIGHT_CACHE/tool"; expected=53 ;;
        tool-offline-missing) export CARGO_NET_OFFLINE=true; rm "$PREFLIGHT_CACHE/tool"; expected=57 ;;
    esac
    status=0
    run "$mode" > "$fixture/$scenario.log" 2>&1 || status=$?
    [[ "$status" == "$expected" ]] || exit 1
    expected_fetch='fetch --locked'
    [[ "$mode" != 0 ]] || expected_fetch+=' --offline'
    printf 'cargo:%s:%s\n' "$PWD" "$expected_fetch" > "$fixture/expected"
    case "$scenario" in
        standalone-offline|environment-offline|config-offline|fetch-failure) [[ ! -e "$PREFLIGHT_CACHE/input" ]] || exit 1 ;;
        tool-standalone-missing) printf 'make:offline-check\n' >> "$fixture/expected" ;;
        tool-install-failure|tool-offline-missing) printf 'make:install\n' >> "$fixture/expected" ;;
        *) printf 'make:install\nmake:offline-check\n' >> "$fixture/expected" ;;
    esac
    cmp "$fixture/expected" "$PREFLIGHT_EVENTS"
    if [[ "$expected" == 0 ]]; then
        [[ -f "$PREFLIGHT_CACHE/input" && -f "$PREFLIGHT_CACHE/tool" ]] || exit 1
    fi
    [[ "$(cat "$PREFLIGHT_CACHE/old-tool")" == 'old executable' ]] || exit 1
    cmp "$fixture/original.toml" Cargo.toml
    cmp "$fixture/original.lock" Cargo.lock
); done
# Each matrix owns its own environment; do not reuse a subshell selection.
unset PREFLIGHT_CACHE CARGO_NET_OFFLINE PREFLIGHT_FETCH_STATUS PREFLIGHT_FAIL_TARGET
# Qualify the real runner/helper boundary here. A successful cold preflight
# reaches only a deliberately failing validation; every setup refusal stops
# earlier. Real Git objects and a bare destination belong solely to this fixture.
export PREFLIGHT_SHARED_ROOT="$ROOT" PREFLIGHT_FIXTURE_MAKE="$fixture/bin/make"
cat > "$fixture/release-make" <<'ADAPTER'
#!/usr/bin/env bash
set -euo pipefail
if [[ $# == 4 && "$1" == --no-print-directory && "$2" == -C && "$3" == "$PREFLIGHT_TOOL_ROOT" ]]; then
    exec "$PREFLIGHT_FIXTURE_MAKE" "$@"
fi
target=''
for argument in "$@"; do
    case "$argument" in
        RELEASE_*=*) export "$argument" ;;
        release-*) target="$argument" ;;
    esac
done
case "$target" in
    release-version) cat version ;;
    release-preflight)
        exec bash "$PREFLIGHT_SHARED_ROOT/scripts/ci/cargo-release-preflight.sh" "$PREFLIGHT_TOOL_ROOT" 1 \
            --pin "$PREFLIGHT_SECOND_PIN" --check offline-check --prepare install --prepare offline-check ;;
    release-verify) printf 'validation\n' >> "$PREFLIGHT_EVENTS"; exit 83 ;;
    '') exec "$PREFLIGHT_REAL_MAKE" "$@" ;;
    *) echo "unexpected release phase: $target" >&2; exit 99 ;;
esac
ADAPTER
chmod +x "$fixture/release-make"
mkdir "$fixture/git-templates"
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TEMPLATE_DIR="$fixture/git-templates"
git init --bare --quiet "$fixture/destination.git"
# This matrix initializes every selection; no prior subshell value is reused.
# shellcheck disable=SC2031
for scenario in cold offline fetch-failure tool-install-failure tool-check-failure; do (
    unset CARGO_NET_OFFLINE PREFLIGHT_FETCH_STATUS PREFLIGHT_FAIL_TARGET
    workspace="$fixture/runner-$scenario"
    mkdir "$workspace"
    cd "$workspace"
    git init --quiet
    git symbolic-ref HEAD refs/heads/main
    printf '0.1.1\n' > version
    printf 'unchanged manifest\n' > Cargo.toml
    printf 'unchanged lock\n' > Cargo.lock
    git add -- version Cargo.toml Cargo.lock
    tree="$(git write-tree)"
    source_commit="$(printf 'Isolated runner preflight fixture\n' | \
        GIT_AUTHOR_NAME=Fixture GIT_AUTHOR_EMAIL=fixture@example.invalid \
        GIT_COMMITTER_NAME=Fixture GIT_COMMITTER_EMAIL=fixture@example.invalid git commit-tree "$tree")"
    git update-ref HEAD "$source_commit"
    git remote add origin "$fixture/destination.git"
    cp .git/index "$fixture/runner-index"
    export PREFLIGHT_CACHE="$fixture/runner-cache-$scenario"
    mkdir "$PREFLIGHT_CACHE"
    printf 'old executable\n' > "$PREFLIGHT_CACHE/old-tool"
    case "$scenario" in
        offline) export CARGO_NET_OFFLINE=true ;;
        fetch-failure) export PREFLIGHT_FETCH_STATUS=47 ;;
        tool-install-failure) export PREFLIGHT_FAIL_TARGET=install ;;
        tool-check-failure) export PREFLIGHT_FAIL_TARGET=offline-check ;;
    esac
    : > "$PREFLIGHT_EVENTS"
    status=0
    RELEASE_MAKE="$fixture/release-make" bash "$ROOT/scripts/ci/run-release.sh" patch origin main \
        > "$workspace.log" 2>&1 || status=$?
    expected=1
    [[ "$scenario" != cold ]] || expected=83
    [[ "$status" == "$expected" ]] || exit 1
    printf 'cargo:%s:fetch --locked\n' "$PWD" > "$fixture/expected"
    case "$scenario" in
        cold) printf 'make:install\nmake:offline-check\nvalidation\n' >> "$fixture/expected" ;;
        tool-install-failure) printf 'make:install\n' >> "$fixture/expected" ;;
        tool-check-failure) printf 'make:install\nmake:offline-check\n' >> "$fixture/expected" ;;
    esac
    cmp "$fixture/expected" "$PREFLIGHT_EVENTS"
    [[ "$(cat version)" == 0.1.1 && "$(git rev-parse HEAD)" == "$source_commit" ]]
    [[ "$(cat Cargo.toml)" == 'unchanged manifest' && "$(cat Cargo.lock)" == 'unchanged lock' ]]
    [[ "$(cat "$PREFLIGHT_CACHE/old-tool")" == 'old executable' ]]
    case "$scenario" in
        offline|fetch-failure) [[ ! -e "$PREFLIGHT_CACHE/input" ]] ;;
        *) [[ -f "$PREFLIGHT_CACHE/input" ]] ;;
    esac
    cmp .git/index "$fixture/runner-index"
    for plan in .git/release-state/*.plan; do [[ ! -e "$plan" ]] || exit 1; done
    [[ ! -e .git/release-state/lock && -z "$(git tag --list)" ]]
    [[ -z "$(git ls-remote --refs origin)" ]]
); done
# Real Cargo must honor both offline surfaces before a tool effect. Use a frozen
# one-dependency workspace, independent of consumer history, source caches and
# dependency graphs. Neither case prepares sources or accesses the network.
for policy in environment config; do (
    unset CARGO_NET_OFFLINE PREFLIGHT_FETCH_STATUS PREFLIGHT_CACHE
    export PREFLIGHT_REAL_FETCH=1 CARGO_HOME="$fixture/real-cargo-home-$policy"
    workspace="$fixture/real-cargo-$policy"
    mkdir -p "$CARGO_HOME" "$workspace/src"
    cat > "$workspace/Cargo.toml" <<'TOML'
[package]
name = "offline-policy-fixture"
version = "0.0.0"
edition = "2021"
[workspace]
[dependencies]
cfg-if = "=1.0.5"
TOML
    printf '#![no_std]\n' > "$workspace/src/lib.rs"
    cat > "$workspace/Cargo.lock" <<'LOCK'
version = 4

[[package]]
name = "cfg-if"
version = "1.0.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4e7648175b45a9a48536d676f68d918270699102aa8dab5496df06904c914600"

[[package]]
name = "offline-policy-fixture"
version = "0.0.0"
dependencies = [
 "cfg-if",
]
LOCK
    cp "$workspace/Cargo.toml" "$workspace/original.toml"
    cp "$workspace/Cargo.lock" "$workspace/original.lock"
    if [[ "$policy" == config ]]; then printf '[net]\noffline = true\n' > "$CARGO_HOME/config.toml"; fi
    cd "$workspace"
    : > "$PREFLIGHT_EVENTS"
    status=0
    if [[ "$policy" == environment ]]; then
        CARGO_NET_OFFLINE=true run 1 > "$workspace/preflight.log" 2>&1 || status=$?
    else
        run 1 > "$workspace/preflight.log" 2>&1 || status=$?
    fi
    [[ "$status" == 101 ]] || exit 1
    printf 'cargo:%s:fetch --locked\n' "$PWD" > "$workspace/expected-events"
    cmp "$workspace/expected-events" "$PREFLIGHT_EVENTS"
    cmp original.toml Cargo.toml
    cmp original.lock Cargo.lock
); done
fixture_complete=true
echo 'Locked Cargo preparation, ordered tool checks and multi-pin refusal passed'
