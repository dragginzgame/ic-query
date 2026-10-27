#!/usr/bin/env bash
set -euo pipefail

repo_root="$(CDPATH="" cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-dependency-review.XXXXXX")"
work_dir="$(CDPATH="" cd -P -- "$work_dir" && printf '%s/.' "$PWD")"
work_dir="${work_dir%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "Dependency review fixtures retained"' EXIT
export TMPDIR="${work_dir%/*}"

fail() {
  fixture_replay_logs "$work_dir/$review_date.log"
  echo "error: dependency review case $review_date: $*" >&2
  exit 1
}

mkdir -p "$work_dir/bin" "$work_dir/adapter"
cat > "$work_dir/bin/date" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$FIXTURE_REVIEW_DATE"
STUB
cat > "$work_dir/bin/git" <<'STUB'
#!/usr/bin/env bash
printf 'fetch\n' >> "$TRACE_FILE"
echo 'dependency review preparation failure' >&2
exit 53
STUB
for tool in 'selected audit' 'selected machete'; do
  cat > "$work_dir/bin/$tool" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "${0##*/}" >> "$TRACE_FILE"
exit 91
STUB
done
chmod +x "$work_dir/bin/"*

run_case() {
  status=0
  TMPDIR="$work_dir/adapter" PATH="$work_dir/bin:$PATH" \
    FIXTURE_REVIEW_DATE="$review_date" TRACE_FILE="$work_dir/trace" \
    "$BASH" "$repo_root/scripts/ci/check-dependencies.sh" \
    "$work_dir/bin/selected audit" "$work_dir/bin/selected machete" \
    > "$work_dir/$review_date.log" 2>&1 || status=$?
  printf '%s\n' "$status" > "$work_dir/$review_date.status"
}

for review_date in 2026-11-07 2026-11-08 invalid; do
  run_case
  [[ "$status" == 2 ]] || fail "returned $status instead of expiry refusal status 2"
  grep -Fq 'acceptance requires review' "$work_dir/$review_date.log" \
    || fail 'expiry refusal lost its diagnostic'
  [[ ! -e "$work_dir/trace" ]] || fail 'refused admission dispatched preparation or audit tools'
  [[ -z "$(find "$work_dir/adapter" -mindepth 1 -print -quit)" ]] \
    || fail 'refused admission allocated preparation state'
done

# The preceding day admits explicit preparation; its original failure must survive.
review_date=2026-11-06
run_case
[[ "$status" == 53 ]] || fail "preparation failure returned $status instead of 53"
printf 'fetch\n' > "$work_dir/expected-trace"
cmp "$work_dir/expected-trace" "$work_dir/trace" \
  || fail 'failed preparation dispatched audit or machete'
preparation_logs=("$work_dir/adapter/"ic-query-dependency-check.*/rustsec/prepare.log)
[[ "${#preparation_logs[@]}" == 1 && -f "${preparation_logs[0]}" ]] \
  || fail 'preparation failure discarded its evidence'
grep -Fq 'dependency review preparation failure' "${preparation_logs[0]}" \
  || fail 'retained preparation evidence lost its diagnostic'

echo 'Dependency review dates, refusal diagnostics and preparation-failure retention passed'
fixture_complete=true
