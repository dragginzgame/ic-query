#!/usr/bin/env bash
set -euo pipefail

repo_root="$(CDPATH="" cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-retained-evidence.XXXXXX")"
work_dir="$(CDPATH="" cd -P -- "$work_dir" && printf '%s/.' "$PWD")"
work_dir="${work_dir%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "Retained evidence fixtures retained"' EXIT
export TMPDIR="${work_dir%/*}"

fixture="$work_dir/repository"
mkdir -p "$fixture/scripts/ci" "$work_dir/bin"
cp "$repo_root/scripts/ci/retain-tool-evidence.sh" \
  "$repo_root/scripts/ci/archive-evidence.sh" "$fixture/scripts/ci/"
cat > "$fixture/scripts/ci/collect-tool-evidence.sh" <<'STUB'
#!/usr/bin/env bash
printf 'tool evidence\n' > "$1"
exit "${COLLECT_STATUS:-0}"
STUB
cat > "$work_dir/bin/tar" <<'STUB'
#!/usr/bin/env bash
if [[ "${FAIL_ARCHIVE:-0}" == 1 ]]; then
  printf 'partial archive\n'
  exit 31
fi
exec "$REAL_TAR" "$@"
STUB
chmod +x "$work_dir/bin/tar"
real_tar="$(command -v tar)"
unusual=$'caller-owned\nfile.txt'
printf 'original fixture evidence\n' > "$work_dir/expected"

for mode in success archive-failure tool-failure both-failures symlinked-tools; do
  case_root="$work_dir/$mode"
  temp_root="$case_root/temp"
  repository="$case_root/repository"
  mkdir -p "$temp_root/ic-query-validation/case" "$temp_root/formatting.fixture" \
    "$repository/target/validation-failures" "$case_root/extracted"
  cp "$work_dir/expected" "$temp_root/ic-query-validation/case/$unusual"
  cp "$work_dir/expected" "$temp_root/formatting.fixture/log"
  cp "$work_dir/expected" "$repository/target/validation-failures/log"
  archive_failure=0
  collect_status=0
  case "$mode" in
    archive-failure) archive_failure=1 ;;
    tool-failure) collect_status=23 ;;
    both-failures) archive_failure=1; collect_status=23 ;;
    symlinked-tools) ln -s "$work_dir/no-tools" "$repository/.tools" ;;
  esac
  status=0
  REAL_TAR="$real_tar" PATH="$work_dir/bin:$PATH" FAIL_ARCHIVE="$archive_failure" \
    COLLECT_STATUS="$collect_status" "$BASH" "$fixture/scripts/ci/retain-tool-evidence.sh" \
    "$repository" "$temp_root" > "$case_root/output.log" 2>&1 || status=$?
  expected_status=0
  case "$mode" in archive-failure|both-failures) expected_status=1 ;; tool-failure) expected_status=23 ;; esac
  [[ "$status" == "$expected_status" ]] || { cat "$case_root/output.log" >&2; exit 1; }
  evidence=("$temp_root/"ic-query-tool-evidence.*)
  [[ "${#evidence[@]}" == 1 && -d "${evidence[0]}" ]] || exit 1
  cmp "$work_dir/expected" "$temp_root/ic-query-validation/case/$unusual"
  if [[ "$archive_failure" == 1 ]]; then
    grep -Fxq 'partial archive' "${evidence[0]}/validation.tar.gz"
  else
    "$real_tar" -xzf "${evidence[0]}/validation.tar.gz" -C "$case_root/extracted"
    cmp "$work_dir/expected" "$case_root/extracted/ic-query-validation/case/$unusual"
    cmp "$work_dir/expected" "$case_root/extracted/formatting.fixture/log"
    cmp "$work_dir/expected" "$case_root/extracted/target/validation-failures/log"
  fi
  if [[ "$mode" == symlinked-tools ]]; then
    [[ ! -e "${evidence[0]}/tools.tar.gz" ]] || exit 1
  else
    grep -Fxq 'tool evidence' "${evidence[0]}/tools.tar.gz"
  fi
done

echo 'Validation archives preserve unusual names, original evidence and independent failure statuses'
fixture_complete=true
