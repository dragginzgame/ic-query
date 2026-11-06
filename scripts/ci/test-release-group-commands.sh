#!/usr/bin/env bash
# Shared companions: Makefile scripts/ci/package-workspace.sh scripts/ci/check-release-source.sh scripts/ci/prepare-script-fixture.sh scripts/ci/fixture-lifecycle.sh
set -euo pipefail
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS
root="$(CDPATH='' cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
mkdir -p "${TMPDIR:-$root/target/tmp}"
work="$(mktemp -d "${TMPDIR:-$root/target/tmp}/ic-query-group-commands.XXXXXX")"
work="$(CDPATH='' cd -P -- "$work" && pwd -P)"
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$work" "Group command fixtures retained"' EXIT
fail() { echo "Query group command fixture failed: $*" >&2; exit 1; }
mkdir "$work/source" "$work/bin"
bash "$root/scripts/ci/prepare-script-fixture.sh" "$root" "$work/source" \
  Makefile scripts/ci/package-workspace.sh scripts/ci/check-release-source.sh
cat > "$work/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo %s\n' "$*" >> "$TRACE_FILE"
[[ " $* " == *' --locked '* && " $* " == *' --offline '* ]] || exit 58
[[ "${FAIL_CARGO:-}" != yes ]] || exit 57
STUB
cat > "$work/bin/git" <<'STUB'
#!/usr/bin/env bash
case "$*" in
  'rev-parse --show-prefix') ;;
  'status --porcelain=v1 -z --untracked-files=all')
    [[ "${DIRTY_SOURCE:-}" != yes ]] || printf ' M Cargo.lock\0' ;;
  *) exit 59 ;;
esac
STUB
chmod +x "$work/bin/"{cargo,git}
export PATH="$work/bin:$PATH" TRACE_FILE="$work/trace"
cd "$work/source"
cat > "$work/query-clippy" <<'EXPECTED'
cargo clippy -p ic-query --all-targets --no-default-features --locked --offline -- -D warnings
cargo clippy -p ic-query -p ic-query-testkit --all-targets --all-features --locked --offline -- -D warnings
EXPECTED
cat > "$work/cli-clippy" <<'EXPECTED'
cargo clippy -p ic-query-cli -p ic-query-text --all-targets --all-features --locked --offline -- -D warnings
EXPECTED
cat > "$work/query-package" <<'EXPECTED'
cargo package -p ic-query --locked --offline
EXPECTED
cat > "$work/cli-package" <<'EXPECTED'
cargo package -p ic-query-text --locked --config patch.crates-io.ic-query.path="crates/ic-query" --offline
cargo package -p ic-query-cli --locked --config patch.crates-io.ic-query.path="crates/ic-query" --config patch.crates-io.ic-query-text.path="crates/ic-query-text" --offline
EXPECTED
for group in query cli; do
  for stage in clippy package; do
    target="$group-$stage"
    : > "$TRACE_FILE"
    make --no-print-directory "$target" > "$work/$target.log" 2>&1
    cmp "$work/$target" "$TRACE_FILE" || fail "$target selected unrelated work or changed Cargo policy"
    : > "$TRACE_FILE"
    if FAIL_CARGO=yes make --no-print-directory "$target" > "$work/$target-failed.log" 2>&1; then
      fail "$target admitted a failed Cargo command"
    fi
    head -1 "$work/$target" > "$work/first-command"
    cmp "$work/first-command" "$TRACE_FILE" || fail "$target continued after failure"
  done
  : > "$TRACE_FILE"
  if DIRTY_SOURCE=yes make --no-print-directory "$group-package" > "$work/$group-dirty.log" 2>&1; then
    fail "$group packaging admitted dirty source"
  fi
  [[ ! -s "$TRACE_FILE" ]] || fail "$group packaging dispatched before source admission"
done
: > "$TRACE_FILE"
if bash scripts/ci/package-workspace.sh --group unknown --offline > "$work/unknown.log" 2>&1; then
  fail 'unknown package group was admitted'
fi
[[ ! -s "$TRACE_FILE" ]] || fail 'unknown package group reached Cargo'
printf 'Query/CLI leaf Cargo selection, archive peers, offline policy and failure admission passed\n'
fixture_complete=true
