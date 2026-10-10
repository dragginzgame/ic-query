#!/usr/bin/env bash
set -euo pipefail

# Exercise only consumer cache dispatch, with no real release or network effects.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS IC_QUERY_RELEASE_PREPARE_CACHE CARGO_NET_OFFLINE
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
make_bin="$(command -v make)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-release-cache.XXXXXX")"
trap 'if [[ $? == 0 ]]; then rm -rf -- "$work_dir"; else echo "Release cache fixtures retained: $work_dir" >&2; fi' EXIT
fail() { echo "release cache fixture failed: $*" >&2; exit 1; }
mkdir -p "$work_dir/repository/"{make,ci,scripts/ci,scripts/dev,scripts/release} "$work_dir/bin"
cd "$work_dir/repository"
cp "$repo_root/Makefile" Makefile
cp "$repo_root/make/"{tools,release,rust-format,execution}.mk make/
cp "$repo_root/scripts/ci/"{check-make-execution,run-formatting}.sh scripts/ci/
cp "$repo_root/scripts/release/adapter.sh" scripts/release/
cp "$repo_root/scripts/ci/next-release-version.sh" scripts/ci/
cp "$repo_root/scripts/ci/check-format-tools.sh" scripts/ci/
printf 'export SHARED_TOOLING_CARGO_SORT_VERSION=2.1.4\n' > ci/tool-versions.env
cat > scripts/ci/run-release.sh <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "${IC_QUERY_RELEASE_PREPARE_CACHE:-}" == 1 ]]
printf '%s\n' "$*" >> "$TRACE_FILE"
exit "${FIXTURE_RUNNER_RESULT:-0}"
STUB
export TRACE_FILE="$work_dir/trace"
for kind in patch minor major; do
  "$make_bin" --no-print-directory "release-$kind"
done
"$make_bin" --no-print-directory release-resume VERSION=0.51.0
printf '%s\n' 'patch origin main' 'minor origin main' 'major origin main' 'resume 0.51.0 origin main' > "$work_dir/expected"
cmp "$TRACE_FILE" "$work_dir/expected"

# Local dispatch must survive inherited or command-line snapshot selections.
mkdir -p "$work_dir/external/scripts/ci"
cat > "$work_dir/external/scripts/ci/run-release.sh" <<'STUB'
printf 'external runner dispatched\n' >> "$TRACE_FILE"
exit 99
STUB
: > "$TRACE_FILE"
"$make_bin" --no-print-directory > "$work_dir/default.log"
[[ ! -s "$TRACE_FILE" ]] || fail 'default goal dispatched a release'
grep -Fq 'Available commands:' "$work_dir/default.log" || fail 'default goal lost help'
for selection in environment command-line; do
  : > "$TRACE_FILE"
  for kind in patch minor major resume; do
    arguments=("release-$kind" VERSION=0.51.0 RELEASE_REMOTE=review RELEASE_BRANCH=review-branch)
    if [[ "$selection" == command-line ]]; then
      arguments+=("SHARED_TOOLING_ROOT=$work_dir/external" IC_QUERY_RELEASE_PREPARE_CACHE=0)
    fi
    SHARED_TOOLING_ROOT="$work_dir/external" IC_QUERY_RELEASE_PREPARE_CACHE=0 \
      "$make_bin" --no-print-directory "${arguments[@]}" > "$work_dir/$selection-$kind.log" 2>&1
  done
  printf '%s\n' 'patch review review-branch' 'minor review review-branch' \
    'major review review-branch' 'resume 0.51.0 review review-branch' > "$work_dir/expected"
  cmp "$TRACE_FILE" "$work_dir/expected" || fail 'selected destination, resume or local cache routing changed'
done
: > "$TRACE_FILE"
status=0
FIXTURE_RUNNER_RESULT=17 "$make_bin" --no-print-directory release-patch \
  > "$work_dir/runner-failure.log" 2>&1 || status=$?
[[ "$status" == 2 ]] || fail 'Make discarded the runner failure'
printf '%s\n' 'patch origin main' > "$work_dir/expected"
cmp "$TRACE_FILE" "$work_dir/expected"
cat > "$work_dir/parent.mk" <<'MAKE'
export SHARED_TOOLING_ROOT := $(EXTERNAL_ROOT)
export IC_QUERY_RELEASE_PREPARE_CACHE := 0
.PHONY: probe checker
probe:
	+$(MAKE) --no-print-directory -C "$(FIXTURE_REPOSITORY)" release-patch
checker:
	+@bash "$(CHECKER)" "$(CONSUMER)" make/tools.mk make/release.mk make/rust-format.mk make/execution.mk scripts/ci/check-make-execution.sh scripts/ci/run-formatting.sh
MAKE
: > "$TRACE_FILE"
"$make_bin" --no-print-directory -f "$work_dir/parent.mk" probe \
  "EXTERNAL_ROOT=$work_dir/external" "FIXTURE_REPOSITORY=$PWD" > "$work_dir/parent.log" 2>&1
cmp "$TRACE_FILE" "$work_dir/expected" || fail 'parent Make redirected the local runner'
: > "$TRACE_FILE"
"$make_bin" --no-print-directory -f "$work_dir/parent.mk" checker \
  "EXTERNAL_ROOT=$work_dir/external" "CHECKER=$repo_root/scripts/ci/check-release-commands.sh" \
  "CONSUMER=$repo_root" > "$work_dir/parent-checker.log" 2>&1
[[ ! -s "$TRACE_FILE" ]] || fail 'release-command checker dispatched the external runner'

cat > "$work_dir/bin/git" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
  'rev-parse --git-path release-state') echo .fixture-state ;;
  'rev-parse HEAD') echo "$RELEASE_SOURCE" ;;
  'diff --name-only -z '*|'diff --cached --name-only -z '*|'ls-files --others --exclude-standard') exit 0 ;;
  *) exit 91 ;;
esac
STUB
cat > scripts/ci/check-release-source.sh <<'STUB'
[[ ! -e dirty-source ]] || exit 42
STUB
cat > scripts/release/metadata.pl <<'STUB'
use strict;
use warnings;
die "preparation permission leaked\n" if exists $ENV{IC_QUERY_RELEASE_PREPARE_CACHE};
my $operation = shift @ARGV;
die "unexpected metadata operation\n" unless $operation =~ /\A(?:preflight|admit|check-paths)\z/;
if ($operation eq 'check-paths') { local $/; my $input = <STDIN>; die "changed paths\n" if length $input; }
STUB
cat > "$work_dir/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ -z "${IC_QUERY_RELEASE_PREPARE_CACHE+x}" ]]
printf '%s\n' "$*" >> "$TRACE_FILE"
case "$*" in
  'sort --version') [[ -f selected-formatter ]]; echo cargo-sort 2.1.4; exit ;;
  'fmt --version') echo rustfmt; exit ;;
esac
[[ "$*" == 'fetch --locked' || "$*" == 'fetch --locked --offline' ]]
if [[ ! -e prepared-cache ]]; then
  if [[ "$*" == *--offline || "${CARGO_NET_OFFLINE:-}" == true ]]; then
    echo 'fixture missing dependency: offline request refused' >&2
    exit 43
  fi
  if [[ "${FIXTURE_NETWORK_FAILURE:-}" == yes ]]; then
    echo 'fixture registry unavailable' >&2
    exit 46
  fi
  : > prepared-cache
fi
STUB
cat > scripts/dev/install-rust-tools.sh <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ -z "${IC_QUERY_RELEASE_PREPARE_CACHE+x}" ]]
expected="--consumer $PWD --package cargo-sort --version 2.1.4 --bin cargo-sort --profile release"
if [[ "$*" == "$expected --check" ]]; then
  printf 'formatter check\n' >> "$TRACE_FILE"
  [[ -f selected-formatter && "${FIXTURE_FORMAT_CHECK_FAILURE:-}" != yes ]] || exit 51
else
  [[ "$*" == "$expected" ]]
  if [[ -f selected-formatter ]]; then
    printf 'formatter reuse\n' >> "$TRACE_FILE"
  else
    printf 'formatter setup\n' >> "$TRACE_FILE"
    [[ "${CARGO_NET_OFFLINE:-}" != true && "${FIXTURE_FORMAT_SETUP_FAILURE:-}" != yes ]] || exit 49
    printf 'admitted formatter\n' > selected-formatter
    [[ "${FIXTURE_FORMAT_CHANGED_INPUT:-}" != yes ]] || : > dirty-source
  fi
fi
STUB
chmod +x "$work_dir/bin/"{git,cargo}
export PATH="$work_dir/bin:$PATH"
export RELEASE_KIND=patch RELEASE_PREVIOUS=0.50.3 RELEASE_VERSION=0.50.4
export RELEASE_DATE=2026-10-09 RELEASE_SOURCE=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
export RELEASE_REMOTE=origin RELEASE_BRANCH=main
printf 'selected lock bytes\n' > Cargo.lock
cp Cargo.lock "$work_dir/original-lock"
refuse() {
  local expected_status="$1" status=0
  : > "$TRACE_FILE"
  bash scripts/release/adapter.sh preflight > "$work_dir/failure.log" 2>&1 || status=$?
  [[ "$status" == "$expected_status" ]] || fail "expected $expected_status, got $status"
  [[ ! -e prepared-cache ]] || fail 'refused preparation changed cache'
  cmp Cargo.lock "$work_dir/original-lock"
}
refuse 43
grep -Fxq 'fetch --locked --offline' "$TRACE_FILE"
grep -Fq 'cargo fetch --locked, then retry' "$work_dir/failure.log"
[[ "$(wc -l < "$TRACE_FILE")" -eq 1 ]] || fail 'offline failure retried'
IC_QUERY_RELEASE_PREPARE_CACHE=1 CARGO_NET_OFFLINE=true refuse 43
grep -Fxq 'fetch --locked' "$TRACE_FILE"
grep -Fq 'offline request refused' "$work_dir/failure.log"
IC_QUERY_RELEASE_PREPARE_CACHE=1 FIXTURE_NETWORK_FAILURE=yes refuse 46
grep -Fq 'fixture registry unavailable' "$work_dir/failure.log"
: > dirty-source
IC_QUERY_RELEASE_PREPARE_CACHE=1 refuse 42
[[ ! -s "$TRACE_FILE" ]] || fail 'dirty source triggered fetching'
rm dirty-source
# A fetched graph alone cannot prepare the selected executable. Tool failures
# refuse before its admission; earlier installations and lock bytes survive.
: > prepared-cache
printf 'older formatter bytes\n' > previous-formatter
cp previous-formatter "$work_dir/previous-formatter"
for selection in offline setup-failure; do
  : > "$TRACE_FILE"
  status=0
  if [[ "$selection" == offline ]]; then
    IC_QUERY_RELEASE_PREPARE_CACHE=1 CARGO_NET_OFFLINE=true \
      bash scripts/release/adapter.sh preflight > "$work_dir/tool-$selection.log" 2>&1 || status=$?
  else
    IC_QUERY_RELEASE_PREPARE_CACHE=1 FIXTURE_FORMAT_SETUP_FAILURE=yes \
      bash scripts/release/adapter.sh preflight > "$work_dir/tool-$selection.log" 2>&1 || status=$?
  fi
  [[ "$status" == 2 && ! -e selected-formatter ]] || fail 'failed formatter setup was admitted'
  printf '%s\n' 'fetch --locked' 'formatter setup' > "$work_dir/expected"
  cmp "$TRACE_FILE" "$work_dir/expected"
  cmp Cargo.lock "$work_dir/original-lock"
  cmp previous-formatter "$work_dir/previous-formatter"
done
status=0
: > "$TRACE_FILE"
IC_QUERY_RELEASE_PREPARE_CACHE=1 FIXTURE_FORMAT_CHANGED_INPUT=yes \
  bash scripts/release/adapter.sh preflight > "$work_dir/tool-changed-input.log" 2>&1 || status=$?
[[ "$status" == 42 && -f selected-formatter ]] || fail 'changed source was admitted after setup'
printf '%s\n' 'fetch --locked' 'formatter setup' > "$work_dir/expected"
cmp "$TRACE_FILE" "$work_dir/expected"
cmp Cargo.lock "$work_dir/original-lock"
cmp previous-formatter "$work_dir/previous-formatter"
rm dirty-source selected-formatter
rm prepared-cache
: > "$TRACE_FILE"
IC_QUERY_RELEASE_PREPARE_CACHE=1 bash scripts/release/adapter.sh preflight
[[ -f prepared-cache ]] || fail 'standard release did not prepare cold cache'
printf '%s\n' 'fetch --locked' 'formatter setup' 'formatter check' 'sort --version' 'fmt --version' > "$work_dir/expected"
cmp "$TRACE_FILE" "$work_dir/expected"
cmp Cargo.lock "$work_dir/original-lock"
cp selected-formatter "$work_dir/selected-formatter"
: > "$TRACE_FILE"
IC_QUERY_RELEASE_PREPARE_CACHE=1 "$make_bin" --no-print-directory -j4 release-preflight > "$work_dir/tool-reuse.log" 2>&1
printf '%s\n' 'fetch --locked' 'formatter reuse' 'formatter check' 'sort --version' 'fmt --version' > "$work_dir/expected"
cmp "$TRACE_FILE" "$work_dir/expected"
if grep -Ei 'jobserver unavailable|jobserver.*forced' "$work_dir/tool-reuse.log"; then fail 'tool preparation lost the Make jobserver'; fi
cmp selected-formatter "$work_dir/selected-formatter"
cmp previous-formatter "$work_dir/previous-formatter"
status=0
: > "$TRACE_FILE"
IC_QUERY_RELEASE_PREPARE_CACHE=1 FIXTURE_FORMAT_CHECK_FAILURE=yes \
  bash scripts/release/adapter.sh preflight > "$work_dir/tool-check-failure.log" 2>&1 || status=$?
[[ "$status" == 2 ]] || fail 'failed formatter admission was discarded'
printf '%s\n' 'fetch --locked' 'formatter reuse' 'formatter check' > "$work_dir/expected"
cmp "$TRACE_FILE" "$work_dir/expected"

# Saved evidence is admitted before cache preparation against partial metadata.
mkdir -p .fixture-state/0.50.4.verify.saved
printf '%s\n' .fixture-state/0.50.4.verify.saved > .fixture-state/0.50.4.validation
printf '%s\n' validation-1 "$RELEASE_KIND" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" \
  "$RELEASE_DATE" "$RELEASE_SOURCE" "$RELEASE_REMOTE" "$RELEASE_BRANCH" > .fixture-state/0.50.4.verify.saved/identity
: > .fixture-state/0.50.4.verify.saved/passed
rm prepared-cache
printf 'interrupted metadata\n' > Cargo.lock
cp Cargo.lock "$work_dir/interrupted-lock"
: > "$TRACE_FILE"
IC_QUERY_RELEASE_PREPARE_CACHE=1 bash scripts/release/adapter.sh preflight
[[ ! -s "$TRACE_FILE" && ! -e prepared-cache ]] || fail 'recovery fetched against interrupted metadata'
cmp Cargo.lock "$work_dir/interrupted-lock"
echo 'IC Query locked release-cache preparation and recovery fixtures passed'
