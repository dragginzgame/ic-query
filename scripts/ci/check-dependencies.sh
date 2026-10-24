#!/usr/bin/env bash
set -euo pipefail

[[ $# == 2 && -x "$1" && -x "$2" ]] || {
  echo 'usage: check-dependencies.sh AUDIT-EXECUTABLE MACHETE-EXECUTABLE' >&2
  exit 2
}
audit="$1"
machete="$2"

# Keep the temporary upstream-alignment acceptance subject to explicit review.
review_date="$(date -u +%Y-%m-%d)"
if [[ ! "$review_date" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ || "$review_date" > 2026-11-06 ]]; then
  echo 'RUSTSEC-2026-0387 acceptance requires review by 2026-11-07; see docs/security-advisories.md' >&2
  exit 2
fi

repo_root="$(CDPATH="" cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-dependency-check.XXXXXX")"
work_dir="$(CDPATH="" cd -P -- "$work_dir" && printf '%s/.' "$PWD")"
work_dir="${work_dir%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "Dependency check failed; evidence retained"' EXIT
export TMPDIR="${work_dir%/*}"

# Always start from one coherent RustSec snapshot. cargo-audit's shared cache
# can retain files removed by upstream advisory moves, causing unrelated parse
# failures before the workspace lockfile is inspected.
echo "Fetching a fresh RustSec snapshot (shallow clone; Git progress follows)" >&2
bash "$repo_root/scripts/ci/prepare-rustsec-db.sh" online \
  https://github.com/RustSec/advisory-db.git "$work_dir/rustsec" > /dev/null

# RUSTSEC-2026-0387: temporary upstream-alignment acceptance; review by
# 2026-11-07. Scope, residual signing risk and removal: docs/security-advisories.md.
"$audit" audit --no-fetch --db "${work_dir}/rustsec/db" --deny warnings \
  --ignore RUSTSEC-2021-0127 \
  --ignore RUSTSEC-2024-0436 \
  --ignore RUSTSEC-2026-0387

# Machete uses CARGO to select Cargo-subcommand argument parsing.
unset CARGO
CARGO_NET_OFFLINE=true "$machete" --with-metadata
fixture_complete=true
