#!/usr/bin/env bash
set -euo pipefail

repo_root="$(CDPATH="" cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-fixture-completion.XXXXXX")"
work_dir="$(CDPATH="" cd -P -- "$work_dir" && printf '%s/.' "$PWD")"
work_dir="${work_dir%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "Fixture completion diagnostics retained"' EXIT
export TMPDIR="${work_dir%/*}"

mkdir -p "$work_dir/scripts/ci" "$work_dir/attempts"
cp "$repo_root/scripts/ci/fixture-lifecycle.sh" "$work_dir/scripts/ci/"
mkdir "$work_dir/redirected-attempts"
printf 'unrelated evidence\n' > "$work_dir/redirected-attempts/payload"
ln -s "$work_dir/attempts" "$work_dir/relative-parent"
printf '#!/usr/bin/env bash\nexit 0\n' > "$work_dir/fixture-tool"
chmod +x "$work_dir/fixture-tool"
for owner in check-ci-scripts check-release-cache check-release-metadata \
  check-release-guards check-publish-guards check-public-docs \
  check-library-feature-boundaries check-dependencies test-dependency-review \
  test-retained-evidence test-fixture-completion; do
  # Execute the actual setup/handler prefix; stop before any validation effects.
  awk '{ print } /^export TMPDIR=/ { found=1; exit } END { if (!found) exit 1 }' \
    "$repo_root/scripts/ci/$owner.sh" > "$work_dir/scripts/ci/$owner.sh"
  cat >> "$work_dir/scripts/ci/$owner.sh" <<'PROBE'
printf '%s\n' "$work_dir" > "$COMPLETION_DIRECTORY"
printf 'retained evidence\n' > "$work_dir/payload"
mkdir "$work_dir/child"
ln -sfn "$COMPLETION_OTHER_PARENT" "$COMPLETION_ALIAS"
cd "$work_dir/child"
[[ "$(cat "$work_dir/payload")" == 'retained evidence' ]] || exit 1
case "$COMPLETION_MODE" in
  nounset) unset COMPLETION_UNDEFINED; printf '%s\n' "$COMPLETION_UNDEFINED" ;;
  nounset-function) unset COMPLETION_UNDEFINED; missing_value() { printf '%s\n' "$COMPLETION_UNDEFINED"; }; missing_value ;;
  command) false ;;
  nonzero) exit 23 ;;
  premature) exit 0 ;;
  success) fixture_complete=true; exit 0 ;;
  completed-failure) fixture_complete=true; exit 23 ;;
  cleanup-failure) rm() { return 37; }; fixture_complete=true; exit 0 ;;
  reporting-failure) printf() { return 41; }; exit 23 ;;
esac
PROBE
  arguments=()
  [[ "$owner" != check-dependencies ]] || arguments=("$work_dir/fixture-tool" "$work_dir/fixture-tool")
  for mode in nounset nounset-function command nonzero premature success completed-failure cleanup-failure reporting-failure; do
    status=0
    ln -sfn "$work_dir/attempts" "$work_dir/relative-parent"
    (
      cd "$work_dir"
      COMPLETION_MODE="$mode" COMPLETION_DIRECTORY="$work_dir/directory" \
        CDPATH="$work_dir" COMPLETION_ALIAS="$work_dir/relative-parent" \
        COMPLETION_OTHER_PARENT="$work_dir/redirected-attempts" \
        TMPDIR=relative-parent "$BASH" "$work_dir/scripts/ci/$owner.sh" ${arguments[@]+"${arguments[@]}"}
    ) > "$work_dir/$owner-$mode.log" 2>&1 || status=$?
    [[ "$(cat "$work_dir/redirected-attempts/payload")" == 'unrelated evidence' ]] || exit 1
    selected="$(cat "$work_dir/directory")"
    case "$selected" in "$work_dir/attempts/"*) ;; *) exit 1 ;; esac
    expected=1
    case "$mode" in success) expected=0 ;; nonzero|completed-failure|reporting-failure) expected=23 ;; cleanup-failure) expected=37 ;; esac
    [[ "$status" == "$expected" ]] || {
      echo "$owner $mode returned $status, expected $expected" >&2; exit 1;
    }
    if [[ "$mode" == success ]]; then
      [[ ! -e "$selected" ]] || exit 1
    else
      [[ "$(cat "$selected/payload")" == 'retained evidence' ]] || exit 1
    fi
  done
done
echo 'Consumer completion, failure status and evidence retention fixtures passed'
fixture_complete=true
