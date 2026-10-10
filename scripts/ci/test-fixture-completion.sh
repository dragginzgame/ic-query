#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-fixture-completion.XXXXXX")"
fixture_complete=false
finish() {
  local status=$?
  [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
  if [[ "$status" == 0 ]]; then rm -rf -- "$work_dir"
  else echo "Fixture completion diagnostics retained: $work_dir" >&2; fi
  exit "$status"
}
trap finish EXIT

mkdir -p "$work_dir/scripts/ci" "$work_dir/attempts"
for owner in check-ci-scripts check-release-cache check-release-metadata \
  check-release-guards check-publish-guards check-public-docs \
  check-library-feature-boundaries check-dependencies test-fixture-completion; do
  # Execute the actual setup/handler prefix; stop before any validation effects.
  awk '{ print } /^trap finish EXIT$/ { found=1; exit } END { if (!found) exit 1 }' \
    "$repo_root/scripts/ci/$owner.sh" > "$work_dir/scripts/ci/$owner.sh"
  cat >> "$work_dir/scripts/ci/$owner.sh" <<'PROBE'
printf '%s\n' "$work_dir" > "$COMPLETION_DIRECTORY"
printf 'retained evidence\n' > "$work_dir/payload"
case "$COMPLETION_MODE" in
  nounset) unset COMPLETION_UNDEFINED; printf '%s\n' "$COMPLETION_UNDEFINED" ;;
  command) false ;;
  nonzero) exit 23 ;;
  premature) exit 0 ;;
  success) fixture_complete=true; exit 0 ;;
  completed-failure) fixture_complete=true; exit 23 ;;
esac
PROBE
  for mode in nounset command nonzero premature success completed-failure; do
    status=0
    COMPLETION_MODE="$mode" COMPLETION_DIRECTORY="$work_dir/directory" \
      TMPDIR="$work_dir/attempts" "$BASH" "$work_dir/scripts/ci/$owner.sh" \
      > "$work_dir/$owner-$mode.log" 2>&1 || status=$?
    selected="$(cat "$work_dir/directory")"
    case "$selected" in "$work_dir/attempts/"*) ;; *) exit 1 ;; esac
    expected=1
    case "$mode" in success) expected=0 ;; nonzero|completed-failure) expected=23 ;; esac
    [[ "$status" == "$expected" ]] || {
      echo "$owner $mode returned $status, expected $expected" >&2; exit 1;
    }
    if [[ "$mode" == success ]]; then
      [[ ! -e "$selected" ]]
    else
      [[ "$(cat "$selected/payload")" == 'retained evidence' ]]
    fi
  done
done
echo 'Consumer completion, failure status and evidence retention fixtures passed'
fixture_complete=true
