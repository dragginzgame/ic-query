#!/usr/bin/env bash
set -euo pipefail

# IC Query owns gates, release files and evidence. The shared runner owns Git effects.
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$repo_root"
operation="${1:-}"
prepare_cache="${IC_QUERY_RELEASE_PREPARE_CACHE:-0}"
unset IC_QUERY_RELEASE_PREPARE_CACHE
metadata() { perl scripts/release/metadata.pl "$@"; }
fail() { echo "release adapter refused: $*" >&2; exit 1; }
[[ "${RELEASE_DELIVERY:-direct}" == direct ]] \
  || fail 'IC Query requires direct release delivery; PR delivery is not qualified'
for selection in RELEASE_KIND RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE RELEASE_SOURCE RELEASE_REMOTE RELEASE_BRANCH; do
  [[ -n "${!selection:-}" ]] || fail "missing $selection"
done
observed_version="$(bash scripts/ci/next-release-version.sh "$RELEASE_PREVIOUS" "$RELEASE_KIND")" || exit $?
[[ "$observed_version" == "$RELEASE_VERSION" ]] \
  || fail 'candidate differs from the selected increment'
if [[ "$operation" == files ]]; then metadata files; exit; fi
state_root="$(git rev-parse --git-path release-state)"
mkdir -p "$state_root"
binding="$state_root/$RELEASE_VERSION.validation"
identity() {
  printf '%s\n' validation-1 "$RELEASE_KIND" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" \
    "$RELEASE_DATE" "$RELEASE_SOURCE" "$RELEASE_REMOTE" "$RELEASE_BRANCH"
}
load_evidence() {
  [[ -f "$binding" && ! -L "$binding" ]] || fail 'successful source-bound validation evidence is missing'
  IFS= read -r evidence < "$binding"
  case "$evidence" in "$state_root/$RELEASE_VERSION.verify."*) ;; *) fail 'invalid evidence path' ;; esac
  [[ -d "$evidence" && ! -L "$evidence" ]] || fail 'validation evidence is missing or symlinked'
  identity | cmp -s - "$evidence/identity" || fail 'validation selections differ from retained evidence'
  [[ -f "$evidence/passed" ]] || fail 'validation did not complete'
}
admit_paths() {
  local changes untracked
  changes="$(mktemp "$state_root/changed-paths.XXXXXX")"
  if [[ "$#" == 1 ]]; then
    git diff --cached --name-only -z "$1" -- > "$changes"
    git diff --name-only -z -- >> "$changes"
  else
    git diff --name-only -z "$@" -- > "$changes"
  fi
  metadata check-paths < "$changes"
  rm "$changes"
  untracked="$(git ls-files --others --exclude-standard)"
  [[ -z "$untracked" ]] || fail 'untracked source remains'
}
prepared_check() {
  load_evidence
  metadata check "$evidence"
  admit_paths "$RELEASE_SOURCE"
  metadata_log="$(mktemp "$evidence/metadata-check.XXXXXX")"
  cargo metadata --locked --offline --no-deps --format-version 1 > "$metadata_log"
}
committed_check() {
  load_evidence
  metadata check-commit "$evidence"
  admit_paths "$RELEASE_SOURCE" "${RELEASE_COMMIT:?}"
}
case "$operation" in
  preflight)
    observed_source="$(git rev-parse HEAD)" || exit $?
    [[ "$observed_source" == "$RELEASE_SOURCE" ]] || fail 'source identity changed'
    if [[ -e "$binding" ]]; then
      load_evidence
      admit_paths "$RELEASE_SOURCE"
      metadata admit "$evidence"
    else
      bash scripts/ci/check-release-source.sh
      metadata preflight
      fetch_args=(--locked)
      [[ "$prepare_cache" == 1 ]] || fetch_args+=(--offline)
      if cargo fetch "${fetch_args[@]}"; then
        :
      else
        status="$?"
        echo 'release dependency preparation failed; prepare the selected cache with cargo fetch --locked, then retry' >&2
        exit "$status"
      fi
      if [[ "$prepare_cache" == 1 ]]; then
        make --no-print-directory install-format-tools
      else
        CARGO_NET_OFFLINE=true make --no-print-directory install-format-tools
      fi
      observed_source="$(git rev-parse HEAD)" || exit $?
      [[ "$observed_source" == "$RELEASE_SOURCE" ]] || fail 'source identity changed during tool preparation'
      bash scripts/ci/check-release-source.sh
      metadata preflight
      make --no-print-directory format-tools-check
    fi
    ;;
  verify)
    attempt="$(mktemp -d "$state_root/$RELEASE_VERSION.verify.XXXXXX")"
    identity > "$attempt/identity"
    if metadata capture "$attempt" > "$attempt/input-check.log" 2>&1; then
      :
    else
      status="$?"
      cat "$attempt/input-check.log" >&2
      exit "$status"
    fi
    CHANGELOG_VERSION="$RELEASE_VERSION" CARGO_NET_OFFLINE=true \
      VALIDATION_FAILURE_LOG_DIR="$attempt/validation-failures" \
      bash scripts/ci/run-validation-targets.sh --fail-fast ci 2>&1 | tee "$attempt/validation.log"
    : > "$attempt/passed"
    temporary_binding="$(mktemp "$binding.tmp.XXXXXX")"
    printf '%s\n' "$attempt" > "$temporary_binding"
    mv "$temporary_binding" "$binding"
    ;;
  prepare)
    load_evidence
    metadata prepare "$evidence"
    ;;
  prepared-check) prepared_check ;;
  commit-check)
    prepared_check
    metadata check-index "$evidence"
    git diff --quiet -- || fail 'unstaged release payload remains'
    ;;
  committed-check|tagged-check|push-check) committed_check ;;
  *) fail 'unknown operation' ;;
esac
