#!/usr/bin/env bash
set -euo pipefail

repo_root="$(CDPATH="" cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-public-docs.XXXXXX")"
work_dir="$(CDPATH="" cd -P -- "$work_dir" && printf '%s/.' "$PWD")"
work_dir="${work_dir%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "Public documentation diagnostics retained"' EXIT
export TMPDIR="${work_dir%/*}"

# Cargo replays cached diagnostics; the parser rejects an incomplete set.
if ! CARGO_TERM_COLOR=never RUSTDOCFLAGS='-W missing-docs' \
  cargo doc -p ic-query --all-features --no-deps --locked --offline --message-format=json \
  >"${work_dir}/diagnostics.jsonl" 2>>"${work_dir}/stderr"; then
  cat "${work_dir}/stderr" "${work_dir}/diagnostics.jsonl" >&2
  exit 1
fi
perl "${repo_root}/scripts/ci/check-public-docs.pl" \
  "${work_dir}/diagnostics.jsonl" "${repo_root}/scripts/ci/public-docs-baseline.json"
fixture_complete=true
