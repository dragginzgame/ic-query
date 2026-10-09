#!/usr/bin/env bash
set -euo pipefail

# Exercise only consumer cache dispatch, with no real release or network effects.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS IC_QUERY_RELEASE_PREPARE_CACHE CARGO_NET_OFFLINE
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
make_bin="$(command -v make)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-release-cache.XXXXXX")"
trap 'if [[ $? == 0 ]]; then rm -rf -- "$work_dir"; else echo "Release cache fixtures retained: $work_dir" >&2; fi' EXIT
fail() { echo "release cache fixture failed: $*" >&2; exit 1; }
mkdir -p "$work_dir/repository/"{make,scripts/ci,scripts/release} "$work_dir/bin"
cd "$work_dir/repository"
cp "$repo_root/Makefile" Makefile
cp "$repo_root/make/tools.mk" make/
cp "$repo_root/scripts/release/adapter.sh" scripts/release/
cp "$repo_root/scripts/ci/next-release-version.sh" scripts/ci/
cat > scripts/ci/run-release.sh <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "${IC_QUERY_RELEASE_PREPARE_CACHE:-}" == 1 ]]
printf '%s\n' "$*" >> "$TRACE_FILE"
STUB
export TRACE_FILE="$work_dir/trace"
for kind in patch minor major; do
  "$make_bin" --no-print-directory "release-$kind"
done
"$make_bin" --no-print-directory release-resume VERSION=0.51.0
printf '%s\n' 'patch origin main' 'minor origin main' 'major origin main' 'resume 0.51.0 origin main' > "$work_dir/expected"
cmp "$TRACE_FILE" "$work_dir/expected"

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
: > "$TRACE_FILE"
IC_QUERY_RELEASE_PREPARE_CACHE=1 bash scripts/release/adapter.sh preflight
[[ -f prepared-cache ]] || fail 'standard release did not prepare cold cache'
grep -Fxq 'fetch --locked' "$TRACE_FILE"
cmp Cargo.lock "$work_dir/original-lock"

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
