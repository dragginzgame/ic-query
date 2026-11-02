#!/usr/bin/env bash
set -euo pipefail
repo_root="$(CDPATH='' cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-compiler.XXXXXX")"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "Compiler fixtures retained"' EXIT
mkdir "$work_dir/bin"
cat > "$work_dir/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == --version ]]; then
  printf 'cargo %s (fixture)\n' "${FIXTURE_CARGO_VERSION:-1.99.1}"
  [[ "${FIXTURE_FAILURE:-}" != cargo ]]
  exit "$?"
fi
[[ "$1" == metadata && "$CARGO_NET_OFFLINE" == true && "$RUSTUP_AUTO_INSTALL" == 0 ]] || exit 50
case " $* " in *' --locked '*) ;; *) exit 51 ;; esac
case " $* " in *' --offline '*) ;; *) exit 52 ;; esac
if [[ -n "${FIXTURE_METADATA:-}" ]]; then
  printf '%s\n' "$FIXTURE_METADATA"
else
  printf '%s\n' '{"workspace_members":["fixture"],"packages":[{"id":"fixture","rust_version":"1.99.1"}]}'
fi
[[ "${FIXTURE_FAILURE:-}" != metadata ]]
STUB
cat > "$work_dir/bin/rustc" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == --version ]] || exit 50
printf 'rustc %s (fixture)\n' "${FIXTURE_RUSTC_VERSION:-1.99.1}"
[[ "${FIXTURE_FAILURE:-}" != rustc ]]
STUB
chmod +x "$work_dir/bin/"{cargo,rustc}
export PATH="$work_dir/bin:$PATH"
check="$repo_root/scripts/ci/check-rust-compiler.sh"
bash "$check" > "$work_dir/accepted.log"
grep -Fxq 'Rust compiler policy: 1.99.1' "$work_dir/accepted.log"
for failure in metadata cargo rustc; do
  if FIXTURE_FAILURE="$failure" bash "$check" > "$work_dir/$failure.log" 2>&1; then
    echo "error: failed $failure observation admitted the compiler" >&2; exit 1
  fi
done
for version in 1.99.0 stable; do
  if FIXTURE_CARGO_VERSION="$version" bash "$check" > "$work_dir/cargo-mismatch.log" 2>&1; then exit 1; fi
  if FIXTURE_RUSTC_VERSION="$version" bash "$check" > "$work_dir/rustc-mismatch.log" 2>&1; then exit 1; fi
done
for metadata in \
  '{"workspace_members":["fixture"],"packages":[{"id":"fixture"}]}' \
  '{"workspace_members":["a","b"],"packages":[{"id":"a","rust_version":"1.99.1"},{"id":"b","rust_version":"1.99.0"}]}' \
  'invalid JSON'; do
  if FIXTURE_METADATA="$metadata" bash "$check" > "$work_dir/metadata-refusal.log" 2>&1; then exit 1; fi
done
echo 'Compiler admission binds inherited package policy and rejects mismatched or failed Cargo/rustc observations'
fixture_complete=true
