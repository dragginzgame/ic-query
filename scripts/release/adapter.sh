#!/usr/bin/env bash
# Shared companions: scripts/release/metadata.pl scripts/ci/next-release-version.sh scripts/ci/check-release-source.sh scripts/ci/cargo-release-preflight.sh scripts/ci/run-validation-targets.sh
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
read_evidence() {
  [[ -f "$binding" && ! -L "$binding" ]] || fail 'successful source-bound validation evidence is missing'
  IFS= read -r evidence < "$binding"
  case "$evidence" in "$state_root/$RELEASE_VERSION.verify."*) ;; *) fail 'invalid evidence path' ;; esac
  [[ -d "$evidence" && ! -L "$evidence" ]] || fail 'validation evidence is missing or symlinked'
  [[ -f "$evidence/passed" ]] || fail 'validation did not complete'
}
load_evidence() {
  read_evidence
  identity | cmp -s - "$evidence/identity" || fail 'validation selections differ from retained evidence'
}
retain_superseded_validation() {
  local saved_source saved_date archive
  saved_source="$(sed -n '6p' "$evidence/identity")"
  saved_date="$(sed -n '5p' "$evidence/identity")"
  [[ "$saved_source" =~ ^[0-9a-f]{40,64}$ && "$saved_source" != "$RELEASE_SOURCE" \
    && "$saved_date" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] \
    || fail 'validation selections differ from retained evidence'
  printf '%s\n' validation-1 "$RELEASE_KIND" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" \
    "$saved_date" "$saved_source" "$RELEASE_REMOTE" "$RELEASE_BRANCH" \
    | cmp -s - "$evidence/identity" || fail 'superseded validation has conflicting selections'
  git merge-base --is-ancestor "$saved_source" "$RELEASE_SOURCE" \
    || fail 'superseded validation source is not an ancestor'
  bash scripts/ci/check-release-source.sh
  metadata preflight
  # Keep the original pointer and immutable attempt before a fresh gate binds it.
  archive="$(mktemp -d "$state_root/$RELEASE_VERSION.validation-attempt.XXXXXX")"
  mv "$binding" "$archive/$RELEASE_VERSION.validation"
  printf 'Retained superseded validation: %s\n' "$archive/$RELEASE_VERSION.validation" >&2
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
check_manifest_order() {
  local selected_root="$repo_root" observation
  if [[ -n "${1:-}" ]]; then
    observation="$(mktemp -d "$state_root/$RELEASE_VERSION.manifest-order.XXXXXX")"
    selected_root="$observation/source"
    mkdir "$selected_root"
    printf '%s\n' "$1" > "$observation/commit"
    git archive --format=tar "$1" | tar -xf - -C "$selected_root"
    printf 'Selected release source retained for manifest ordering: %s\n' "$selected_root" >&2
  fi
  (cd "$selected_root" && "${FORMAT_CARGO:-cargo}" sort --workspace --grouped --check)
}
prepared_check() {
  load_evidence
  metadata check "$evidence"
  check_manifest_order
  admit_paths "$RELEASE_SOURCE"
  metadata_log="$(mktemp "$evidence/metadata-check.XXXXXX")"
  cargo metadata --locked --offline --no-deps --format-version 1 > "$metadata_log"
}
committed_check() {
  load_evidence
  metadata check-commit "$evidence"
  check_manifest_order "${RELEASE_COMMIT:?}"
  admit_paths "$RELEASE_SOURCE" "${RELEASE_COMMIT:?}"
}
case "$operation" in
  preflight)
    observed_source="$(git rev-parse HEAD)" || exit $?
    [[ "$observed_source" == "$RELEASE_SOURCE" ]] || fail 'source identity changed'
    if [[ -e "$binding" || -L "$binding" ]]; then
      read_evidence
      if identity | cmp -s - "$evidence/identity"; then
        admit_paths "$RELEASE_SOURCE"
        metadata admit "$evidence"
        exit
      fi
      retain_superseded_validation
    fi
    bash scripts/ci/check-release-source.sh
    metadata preflight
    preparation=(bash scripts/ci/cargo-release-preflight.sh "$repo_root" "$prepare_cache"
      --pin ci/tool-versions.env --pin "${IC_TOOL_PINS:-ci/ic-tools.tsv}"
      --check install-rust-tools --prepare install-rust-tools)
    [[ "$prepare_cache" == 1 ]] || preparation=(env CARGO_NET_OFFLINE=true "${preparation[@]}")
    if "${preparation[@]}"; then
      :
    else
      status="$?"
      echo 'release preparation failed; prepare the selected cache with cargo fetch --locked, then retry with the selected tools installed' >&2
      exit "$status"
    fi
    observed_source="$(git rev-parse HEAD)" || exit $?
    [[ "$observed_source" == "$RELEASE_SOURCE" ]] || fail 'source identity changed during tool preparation'
    bash scripts/ci/check-release-source.sh
    metadata preflight
    make --no-print-directory format-tools-check
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
      bash scripts/ci/run-validation-targets.sh --fail-fast \
        --input Cargo.toml --input Cargo.lock ci 2>&1 | tee "$attempt/validation.log"
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
