#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-public-docs.XXXXXX")"
# Bash 3.2 may report zero after nounset; success requires completion.
fixture_complete=false
finish() {
  local status=$?
  [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
  if [[ "$status" == 0 ]]; then rm -rf -- "$work_dir"
  else echo "Public documentation diagnostics retained: $work_dir" >&2; fi
  exit "$status"
}
trap finish EXIT

# Cargo replays cached diagnostics; the parser rejects an incomplete set.
if ! CARGO_TERM_COLOR=never RUSTDOCFLAGS='-W missing-docs' \
  cargo doc -p ic-query --all-features --no-deps --locked --offline --message-format=json \
  >"${work_dir}/diagnostics.jsonl" 2>>"${work_dir}/stderr"; then
  cat "${work_dir}/stderr" "${work_dir}/diagnostics.jsonl" >&2
  exit 1
fi
python3 "${repo_root}/scripts/ci/check-public-docs.py" \
  "${work_dir}/diagnostics.jsonl" "${repo_root}/scripts/ci/public-docs-baseline.json"
fixture_complete=true
