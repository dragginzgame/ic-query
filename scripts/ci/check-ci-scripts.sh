#!/usr/bin/env bash
set -euo pipefail

repo_root="$(CDPATH="" cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
make_bin="$(command -v make)"
export CI_FIXTURE_REAL_MAKE="$make_bin"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-ci-scripts.XXXXXX")"
work_dir="$(CDPATH="" cd -P -- "$work_dir" && printf '%s/.' "$PWD")"
work_dir="${work_dir%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$repo_root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$work_dir" "CI script fixtures retained"' EXIT
export TMPDIR="${work_dir%/*}"
bash "$repo_root/scripts/ci/test-fixture-completion.sh"

fail() {
  echo "error: $*" >&2
  exit 1
}

check_completion_assertions() {
  local fixture="$work_dir/completion-assertions" fault status
  local completion_directories payloads payload_file
  mkdir -p "$fixture/bin" "$fixture/repository/scripts/ci"
  cp "$repo_root/scripts/ci/"*.sh "$fixture/repository/scripts/ci/"
  cat > "$fixture/bin/cat" <<'STUB'
#!/usr/bin/env bash
if [[ "$COMPLETION_ASSERTION_FAULT" == payload && $# == 1 && "$1" == */payload ]]; then
  printf 'contradictory evidence\n'
else
  exec "$COMPLETION_REAL_CAT" "$@"
fi
STUB
  cat > "$fixture/bin/rm" <<'STUB'
#!/usr/bin/env bash
if [[ "$COMPLETION_ASSERTION_FAULT" == cleanup && "${COMPLETION_MODE:-}" == success ]]; then
  exit 0
fi
exec "$COMPLETION_REAL_RM" "$@"
STUB
  chmod +x "$fixture/bin/"{cat,rm}
  printf 'retained evidence\n' > "$fixture/expected-payload"
  for fault in payload cleanup; do
    mkdir -p "$fixture/$fault"
    status=0
    COMPLETION_ASSERTION_FAULT="$fault" COMPLETION_REAL_CAT="$(command -v cat)" \
      COMPLETION_REAL_RM="$(command -v rm)" PATH="$fixture/bin:$PATH" \
      TMPDIR="$fixture/$fault" "$BASH" \
      "$fixture/repository/scripts/ci/test-fixture-completion.sh" \
      > "$fixture/$fault.log" 2>&1 || status=$?
    [[ "$status" == 1 ]] || fail "completion checker accepted contradictory $fault evidence"
    completion_directories=("$fixture/$fault/"ic-query-fixture-completion.*)
    [[ "${#completion_directories[@]}" == 1 && -d "${completion_directories[0]}" ]] \
      || fail "completion checker discarded $fault refusal diagnostics"
    payloads=("${completion_directories[0]}/attempts/"*/payload)
    [[ -f "${payloads[0]}" ]] || fail "completion checker discarded $fault payload"
    for payload_file in "${payloads[@]}"; do
      cmp "$fixture/expected-payload" "$payload_file" \
        || fail "completion checker changed original $fault payload bytes"
    done
  done
  echo 'Completion checker rejects contradictory payload and cleanup observations'
}

check_completion_assertions

check_release_assertions() {
  local fixture="$work_dir/release-assertions" status=0 release_directories
  mkdir -p "$fixture/bin" "$fixture/attempts"
  # Reuse actual fixture setup and assertions, without its unrelated test cases.
  awk '
    /^for observation in / { found=1; exit }
    /^repo_root=/ { print "repo_root=\"$RELEASE_ASSERTION_ROOT\""; next }
    /^bash "\$repo_root\/scripts\/ci\// { next }
    { print }
    END { if (!found) exit 1 }
  ' "$repo_root/scripts/ci/check-release-guards.sh" > "$fixture/probe.sh"
  cat >> "$fixture/probe.sh" <<'PROBE'
new_fixture assertion-phase patch
run_release patch
check_complete
fixture_complete=true
PROBE
  cat > "$fixture/bin/tail" <<'STUB'
#!/usr/bin/env bash
if [[ $# == 3 && "$1" == -n && "$2" == 1 && "$3" == .release-state/*.plan ]]; then
  "$RELEASE_ASSERTION_REAL_TAIL" "$@" >> "$RELEASE_ASSERTION_OBSERVATIONS"
  printf 'contradictory phase\n'
else
  exec "$RELEASE_ASSERTION_REAL_TAIL" "$@"
fi
STUB
  chmod +x "$fixture/bin/tail"
  RELEASE_ASSERTION_ROOT="$repo_root" RELEASE_ASSERTION_REAL_TAIL="$(command -v tail)" \
    RELEASE_ASSERTION_OBSERVATIONS="$fixture/observations" \
    TMPDIR="$fixture/attempts" PATH="$fixture/bin:$PATH" "$BASH" "$fixture/probe.sh" \
    > "$fixture/refusal.log" 2>&1 || status=$?
  [[ "$status" == 1 ]] || fail 'release fixture accepted contradictory plan-phase evidence'
  release_directories=("$fixture/attempts/"ic-query-release-guards.*)
  [[ "${#release_directories[@]}" == 1 && -d "${release_directories[0]}" ]] \
    || fail 'release fixture discarded assertion-refusal diagnostics'
  printf 'complete\n' > "$fixture/expected-observation"
  cmp "$fixture/expected-observation" "$fixture/observations" \
    || fail 'release assertion probe changed the actual completion phase'
  cmp <(printf 'retained artifact\n') \
    "${release_directories[0]}/assertion-phase/target/original-artifact" \
    || fail 'release fixture changed prior artifact bytes after refusal'
  echo 'Release fixture rejects contradictory plan-phase observations'
}

check_release_assertions

copy_makefile() {
  mkdir -p "$1"
  bash "$repo_root/scripts/ci/prepare-script-fixture.sh" "$repo_root" "$1" \
    Makefile scripts/ci/check-release-source.sh
}

check_clean_worktree() {
  local fixture="$work_dir/clean-worktree" replacement unusual
  git clone --quiet --shared "$repo_root" "$fixture"
  check_clean_source() {
    "$make_bin" --no-print-directory -C "$fixture" -f "$repo_root/Makefile" ensure-clean \
      > "$work_dir/clean-source.log" 2>&1
  }
  check_clean_source || fail 'Make refused a clean source checkout'
  cp "$fixture/.git/index" "$work_dir/clean-index"
  printf '\n' >> "$fixture/Cargo.lock"
  cp "$fixture/Cargo.lock" "$work_dir/dirty-lock"
  if check_clean_source; then fail 'Make admitted an unstaged lockfile'; fi
  grep -Fq 'unstaged: Cargo.lock' "$work_dir/clean-source.log" \
    || fail 'Make did not identify the changed lockfile'
  cmp "$fixture/.git/index" "$work_dir/clean-index"
  cmp "$fixture/Cargo.lock" "$work_dir/dirty-lock"

  mkdir -p "$work_dir/clean-bin"
  cat > "$work_dir/clean-bin/cargo" <<'STUB'
#!/usr/bin/env bash
printf 'Cargo dispatched\n' >> "$CLEAN_GATE_TRACE"
STUB
  chmod +x "$work_dir/clean-bin/cargo"
  if PATH="$work_dir/clean-bin:$PATH" CLEAN_GATE_TRACE="$work_dir/package-trace" \
    "$make_bin" --no-print-directory -C "$fixture" -f "$repo_root/Makefile" package \
    > "$work_dir/dirty-package.log" 2>&1; then
    fail 'package admitted an unstaged lockfile'
  fi
  [[ ! -e "$work_dir/package-trace" ]] || fail 'dirty source reached Cargo packaging'
  grep -Fq 'unstaged: Cargo.lock' "$work_dir/dirty-package.log" \
    || fail 'package did not identify the changed lockfile'
  git -C "$fixture" checkout-index --force -- Cargo.lock

  replacement="$(git -C "$fixture" rev-parse HEAD:AGENTS.md)"
  git -C "$fixture" update-index --cacheinfo "100644,$replacement,README.md"
  unusual=$'caller-owned\nfile.txt'
  printf 'retained caller evidence\n' > "$fixture/$unusual"
  cp "$fixture/.git/index" "$work_dir/clean-index"
  if check_clean_source; then fail 'Make admitted staged or untracked source'; fi
  grep -Fq 'staged: README.md' "$work_dir/clean-source.log"
  grep -Fq 'unstaged: README.md' "$work_dir/clean-source.log"
  printf '  untracked: %q\n' "$unusual" > "$work_dir/clean-expected"
  grep -Fx -f "$work_dir/clean-expected" "$work_dir/clean-source.log" >/dev/null
  cmp "$fixture/.git/index" "$work_dir/clean-index"
  [[ "$(cat "$fixture/$unusual")" == 'retained caller evidence' ]] \
    || fail 'source admission changed caller evidence'
}

check_clean_worktree

check_make_execution_modes() {
  local fixture="$work_dir/make-execution" mode status
  mkdir -p "$fixture"
  copy_makefile "$fixture"
  cat >> "$fixture/Makefile" <<'MAKE'

.PHONY: execution-validate execution-probe execution-child
execution-validate:
	+@VALIDATION_REPOSITORY_ROOT="$(CURDIR)" bash "$(LOGGER)" execution-probe
execution-probe:
	@printf '%s\n' '$(RELEASE_REMOTE)' >> "$${TRACE_FILE}"
	+$(MAKE) --no-print-directory execution-child
execution-child:
	@printf '%s\n' '$(RELEASE_REMOTE)' >> "$${TRACE_FILE}"
MAKE
  # Exercise Query's actual Make include boundary; Shared owns the mode matrix.
  status=0
  env -u MAKEFLAGS -u MFLAGS -u MAKEOVERRIDES -u GNUMAKEFLAGS \
    TRACE_FILE="$fixture/trace" "$make_bin" --no-print-directory -C "$fixture" \
    -n MAKEFLAGS= execution-probe > "$fixture/blocked.log" 2>&1 || status=$?
  [[ "$status" == 2 && ! -e "$fixture/trace" ]] || fail 'Query Make admitted dry-run execution'
  for mode in --no-print-directory -j2; do
    env -u MAKEFLAGS -u MFLAGS -u MAKEOVERRIDES -u GNUMAKEFLAGS \
      TRACE_FILE="$fixture/trace" "$make_bin" -C "$fixture" "$mode" \
      RELEASE_REMOTE=fixture-origin LOGGER="$repo_root/scripts/ci/run-validation-targets.sh" \
      execution-validate > "$fixture/executed.log" 2>&1
    printf '%s\n' fixture-origin fixture-origin > "$fixture/expected"
    cmp "$fixture/expected" "$fixture/trace" || fail 'Make lost nested selections or execution'
    if grep -Ei 'jobserver unavailable|jobserver.*forced' "$fixture/executed.log"; then
      fail 'Validation lost the inherited Make jobserver'
    fi
    rm "$fixture/trace"
  done
}

check_make_execution_modes

ci_gate_case="${work_dir}/ci-gate"
mkdir -p "${ci_gate_case}/bin"
cat > "${ci_gate_case}/bin/make" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${2:-}" == -f && "${3:-}" == - ]]; then exec "$CI_FIXTURE_REAL_MAKE" "$@"; fi
printf 'make %s\n' "$*" >> "${TRACE_FILE}"
if [[ "${!#}" == test ]]; then
  printf '%s\n' 'test error::tests::passing ... ok' 'test error::tests::ignored ... ignored'
fi
if [[ "${!#}" == "${FAIL_TARGET:-}" ]]; then
  echo 'error: fixture validation diagnostic' >&2
  exit 43
fi
[[ -z "${EXPECTED_CHANGELOG_VERSION:-}" \
  || "${CHANGELOG_VERSION:-}" == "${EXPECTED_CHANGELOG_VERSION}" ]] || exit 42
EOF
chmod +x "${ci_gate_case}/bin/make"
(
  cd "${repo_root}"
  PATH="${ci_gate_case}/bin:${PATH}" TRACE_FILE="${ci_gate_case}/trace" \
    "${make_bin}" --no-print-directory ci
) > "${ci_gate_case}/passed-output" 2>&1
for line in 'test error::tests::passing ... ok' 'test error::tests::ignored ... ignored'; do
  grep -Fxq "$line" "${ci_gate_case}/passed-output" \
    || fail 'make ci labelled a successful or ignored namespaced test as an error'
done
expected_ci_targets=(
  shared-tooling-check
  tools-check
  clippy
  changelog-check
  dependency-pins-check
  dependency-boundary-check
  package-contents-check
  feature-boundary-check
  library-process-boundary-check
  ci-scripts-check
  publish-guards-check
  release-guards-check
  type-docs-check
  doc-links-check
  public-docs-check
  dependency-check
  schema-version-check
  fmt-check
  check
  test
  package
)
for target in "${expected_ci_targets[@]}"; do
  printf 'make --no-print-directory -C %s -- %s\n' "${repo_root}" "${target}"
done > "${ci_gate_case}/expected-trace"
cmp -s "${ci_gate_case}/expected-trace" "${ci_gate_case}/trace" \
  || fail "make ci ran an unexpected target sequence"

for failed_target in tools-check clippy changelog-check dependency-pins-check dependency-boundary-check doc-links-check test; do
  : > "${ci_gate_case}/trace"
  if (
    cd "${repo_root}"
    PATH="${ci_gate_case}/bin:${PATH}" TRACE_FILE="${ci_gate_case}/trace" FAIL_TARGET="${failed_target}" \
      EXPECTED_CHANGELOG_VERSION=0.8.1 CHANGELOG_VERSION=0.8.1 \
      VALIDATION_FAILURE_LOG_DIR="${ci_gate_case}/failure-logs" \
      "${make_bin}" --no-print-directory ci
  ) > "${ci_gate_case}/failed-output" 2>&1; then
    fail "make ci accepted a failed ${failed_target}"
  fi
  for target in "${expected_ci_targets[@]}"; do
    printf 'make --no-print-directory -C %s -- %s\n' "${repo_root}" "${target}"
    [[ "${target}" != "${failed_target}" ]] || break
  done > "${ci_gate_case}/expected-failed-trace"
  cmp -s "${ci_gate_case}/expected-failed-trace" "${ci_gate_case}/trace" \
    || fail "make ci changed its sequence, continued after failed ${failed_target}, or lost the target changelog version"
  grep -Fq "[ERR:$failed_target] error: fixture validation diagnostic" "${ci_gate_case}/failed-output" \
    || fail 'make ci lost a real validation diagnostic'
done
for line in 'test error::tests::passing ... ok' 'test error::tests::ignored ... ignored'; do
  grep -Fxq "[test] $line" "${ci_gate_case}/failure-logs/latest-errors.log" \
    || fail 'make ci discarded or mislabelled namespaced test failure context'
done

# Validate the consumer's configurable goal list before dispatching any recipe.
for invalid_goal in -n --ignore-errors MAKEFLAGS=i VALUE=1; do
  : > "${ci_gate_case}/trace"
  if PATH="${ci_gate_case}/bin:${PATH}" TRACE_FILE="${ci_gate_case}/trace" \
    VALIDATION_LOG_DIR="${ci_gate_case}/refused-logs" \
    "${make_bin}" --no-print-directory -C "${repo_root}" ci \
    "CI_TARGETS=changelog-check ${invalid_goal}" > "${ci_gate_case}/refused-output" 2>&1; then
    fail 'make ci accepted a goal list containing Make controls'
  fi
  [[ ! -s "${ci_gate_case}/trace" && ! -e "${ci_gate_case}/refused-logs" ]] \
    || fail 'make ci dispatched part of an invalid goal list or created run evidence'
  grep -Fq 'validation requires named Make targets' "${ci_gate_case}/refused-output" \
    || fail 'make ci lost goal admission diagnostics'
done

drift_case="$work_dir/input-drift"
git clone --quiet --shared "$repo_root" "$drift_case"
copy_makefile "$drift_case"
cp "$repo_root/scripts/ci/"{run-validation-targets,verify-file-checksum}.sh "$drift_case/scripts/ci/"
mkdir "$drift_case/bin"
cat > "$drift_case/bin/make" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${2:-}" == -f && "${3:-}" == - ]]; then exec "$CI_FIXTURE_REAL_MAKE" "$@"; fi
printf '%s\n' "${!#}" >> "$TRACE_FILE"
if [[ "${!#}" == ci ]]; then
  printf '\nfixture input edit\n' >> "$DRIFT_INPUT"
  exec "$CI_FIXTURE_REAL_MAKE" "$@"
fi
printf '\nfixture input edit\n' >> "$DRIFT_INPUT"
EOF
chmod +x "$drift_case/bin/make"
for input in Cargo.lock crates/ic-query/Cargo.toml ci/tool-versions.env rust-toolchain.toml ci/ic-tools.tsv ci/cargo-dependency-boundaries.json; do
  cp "$drift_case/$input" "$drift_case/original-input"
  : > "$drift_case/trace"
  if PATH="$drift_case/bin:$PATH" TRACE_FILE="$drift_case/trace" \
    IC_TOOL_PINS="$drift_case/ci/ic-tools.tsv" \
    DRIFT_INPUT="$drift_case/$input" VALIDATION_FAILURE_LOG_DIR="$drift_case/failures" \
    "$make_bin" --no-print-directory -C "$drift_case" ci > "$drift_case/drift-output" 2>&1; then
    fail "CI accepted changed $input"
  fi
  printf 'shared-tooling-check\n' > "$drift_case/expected"
  cmp "$drift_case/expected" "$drift_case/trace" \
    || fail 'CI did not dispatch exactly the first stage before detecting input drift'
  grep -Fq "validation input changed: $input" "$drift_case/failures/latest.log" \
    || fail 'CI lost changed-input diagnostics'
  cp "$drift_case/original-input" "$drift_case/$input"
done
: > "$drift_case/trace"
if (
  cd "$drift_case"
  unset VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH
  PATH="$drift_case/bin:$PATH" TRACE_FILE="$drift_case/trace" DRIFT_INPUT="$drift_case/Cargo.lock" \
    IC_TOOL_PINS="$drift_case/ci/ic-tools.tsv" \
    VALIDATION_FAILURE_LOG_DIR="$drift_case/failures" \
    bash scripts/ci/run-validation-targets.sh --input Cargo.lock ci
) > "$drift_case/nested-output" 2>&1; then
  fail 'nested CI refreshed its inherited input checkpoint'
fi
printf 'ci\n' > "$drift_case/expected"
cmp "$drift_case/expected" "$drift_case/trace" || fail 'nested CI dispatched after inherited input drift'
grep -Fq 'validation input changed: Cargo.lock' "$drift_case/failures/latest.log" \
  || fail 'nested CI lost inherited-input diagnostics'
echo 'Query CI stops subsequent stages on prepared-input drift and retains nested checkpoints'

pin_check_case="${work_dir}/pin-check"
mkdir -p "$pin_check_case/scripts/ci"
copy_makefile "$pin_check_case"
cat > "$pin_check_case/scripts/ci/check-dependency-pins.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ "$#" == 1 && "$1" == --cargo-inheritance ]] || exit 56
[[ "$YQ" == "$EXPECTED_YQ" ]] || exit 56
[[ "${FAIL_PIN_CHECK:-no}" != yes ]] || exit 55
EOF
EXPECTED_YQ="$repo_root/.tools/host/bin/yq" \
  "$make_bin" --no-print-directory -C "$pin_check_case" dependency-pins-check \
  YQ="$repo_root/.tools/host/bin/yq" >/dev/null \
  || fail "dependency-pins-check did not select inheritance checks and the local parser"
if EXPECTED_YQ="$repo_root/.tools/host/bin/yq" FAIL_PIN_CHECK=yes \
  "$make_bin" --no-print-directory -C "$pin_check_case" dependency-pins-check \
  YQ="$repo_root/.tools/host/bin/yq" >/dev/null 2>&1; then
  fail "dependency-pins-check accepted a failed inheritance check"
fi

version_case="$work_dir/version-read"
mkdir -p "$version_case/scripts/ci" "$version_case/bin"
copy_makefile "$version_case"
cp "$repo_root/scripts/ci/read-cargo-workspace-version.sh" "$version_case/scripts/ci/"
printf '[workspace.package]\nversion = "0.1.2" # fixture comment\n[workspace.metadata.fixture]\nversion = "9.9.9"\n' > "$version_case/Cargo.toml"
version_parser="${YQ:-$repo_root/.tools/host/bin/yq}"
actual="$("$make_bin" --no-print-directory -s -C "$version_case" version YQ="$version_parser")" \
  || fail "make version rejected the shared reader's valid manifest"
[[ "$actual" == 0.1.2 && ! -e "$version_case/Cargo.lock" ]] \
  || fail "make version selected another field or resolved dependencies"
cp "$version_case/Cargo.toml" "$version_case/original.toml"
cat > "$version_case/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == locate-project ]]; then
  [[ "$CARGO_NET_OFFLINE" == true && "$RUSTUP_AUTO_INSTALL" == 0 ]] || exit 53
  if [[ "$FAIL_VERSION_CARGO" == yes ]]; then printf '%s\n' "$PWD/Cargo.toml"; exit 52; fi
  exec "$VERSION_REAL_CARGO" "$@"
fi
echo cargo-effect >> "$VERSION_EFFECTS"
exit 43
EOF
cat > "$version_case/failed-parser" <<'EOF'
#!/usr/bin/env bash
printf '{"workspace":{"package":{"version":"0.1.2"}}}\n'
exit 51
EOF
for tool in git curl; do
  cat > "$version_case/bin/$tool" <<'EOF'
#!/usr/bin/env bash
echo external-effect >> "$VERSION_EFFECTS"
exit 43
EOF
done
chmod +x "$version_case/bin/"* "$version_case/failed-parser"
version_fake_path="$version_case/bin:$PATH"
for failure in cargo parser malformed; do
  cp "$version_case/original.toml" "$version_case/Cargo.toml"
  selected_parser="$version_parser"
  [[ "$failure" != parser ]] || selected_parser="$version_case/failed-parser"
  [[ "$failure" != malformed ]] || printf 'version = "0.1.3"\n' >> "$version_case/Cargo.toml"
  for boundary in make tag publish preflight; do
    if (
      cd "$version_case"
      export VERSION_REAL_CARGO VERSION_EFFECTS FAIL_VERSION_CARGO YQ
      VERSION_REAL_CARGO="$(command -v cargo)"
      VERSION_EFFECTS="$version_case/effects"
      FAIL_VERSION_CARGO=no
      [[ "$failure" != cargo ]] || FAIL_VERSION_CARGO=yes
      YQ="$selected_parser"
      case "$boundary" in
        make) PATH="$version_fake_path" "$make_bin" --no-print-directory -s version ;;
        tag) PATH="$version_fake_path" bash "$repo_root/scripts/release/check-tag-at-head.sh" ;;
        publish) PATH="$version_fake_path" bash "$repo_root/scripts/release/publish-workspace.sh" ;;
        preflight) PATH="$version_fake_path" RELEASE_PREVIOUS=0.1.2 RELEASE_VERSION=0.1.3 RELEASE_DATE=2026-10-06 \
          perl "$repo_root/scripts/release/metadata.pl" preflight ;;
      esac
    ) > "$version_case/stdout" 2> "$version_case/stderr"; then
      fail "$boundary accepted a failed $failure version observation"
    fi
    [[ ! -s "$version_case/stdout" && ! -e "$version_case/effects" ]] \
      || fail "$boundary emitted an accepted version or reached effects after failed $failure observation"
  done
done

offline_validation_case="${work_dir}/offline-validation"
mkdir -p "$offline_validation_case/bin" "$offline_validation_case/scripts/ci"
copy_makefile "$offline_validation_case"
cp "$repo_root/scripts/ci/package-workspace.sh" "$offline_validation_case/scripts/ci/"
cat > "$offline_validation_case/bin/git" <<'EOF'
#!/usr/bin/env bash
case "$*" in
  'rev-parse --show-prefix' | 'status --porcelain=v1 -z --untracked-files=all') exit 0 ;;
  *) exit 59 ;;
esac
EOF
cat > "$offline_validation_case/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
case " $* " in *' --locked '*) ;; *) exit 58 ;; esac
case " $* " in *' --offline '*) ;; *) exit 58 ;; esac
if [[ "$1" == metadata ]]; then
  [[ "$CARGO_NET_OFFLINE" == true && "$RUSTUP_AUTO_INSTALL" == 0 ]] || exit 54
  if [[ -n "${MSRV_FIXTURE_METADATA:-}" ]]; then printf '%s\n' "$MSRV_FIXTURE_METADATA"
  else printf '%s\n' '{"workspace_members":["fixture"],"packages":[{"id":"fixture","rust_version":"1.87.0"}]}'
  fi
  [[ "${FAIL_MSRV_METADATA:-}" != yes ]] || exit 53
  exit 0
fi
printf 'cargo %s\n' "$*" >> "$TRACE_FILE"
[[ "${FAIL_CARGO:-}" != yes ]] || exit 57
EOF
chmod +x "$offline_validation_case/bin/"{git,cargo}
for target in build check clippy test msrv package; do
  : > "$offline_validation_case/trace"
  PATH="$offline_validation_case/bin:$PATH" TRACE_FILE="$offline_validation_case/trace" \
    CARGO_NET_OFFLINE=false MAKEFLAGS='' MAKEOVERRIDES='' \
    "$make_bin" --no-print-directory -C "$offline_validation_case" "$target" >/dev/null \
    || fail "$target did not select locked/offline Cargo validation"
  case "$target" in clippy | package) expected_invocations=2 ;; *) expected_invocations=1 ;; esac
  [[ "$(wc -l < "$offline_validation_case/trace")" -eq "$expected_invocations" ]] \
    || fail "$target changed its Cargo validation coverage"
  : > "$offline_validation_case/trace"
  if PATH="$offline_validation_case/bin:$PATH" TRACE_FILE="$offline_validation_case/trace" \
    CARGO_NET_OFFLINE=false FAIL_CARGO=yes MAKEFLAGS='' MAKEOVERRIDES='' \
    "$make_bin" --no-print-directory -C "$offline_validation_case" "$target" >/dev/null 2>&1; then
    fail "$target accepted failed Cargo validation"
  fi
  [[ "$(wc -l < "$offline_validation_case/trace")" -eq 1 ]] \
    || fail "$target continued after failed Cargo validation"
done

PATH="$offline_validation_case/bin:$PATH" TRACE_FILE="$offline_validation_case/default-msrv-trace" \
  MAKEFLAGS='' MAKEOVERRIDES='' "$make_bin" --no-print-directory -s -C "$offline_validation_case" msrv
grep -Fxq 'cargo +1.87.0 check --workspace --all-targets --all-features --locked --offline' \
  "$offline_validation_case/default-msrv-trace" \
  || fail 'MSRV validation did not use the manifest metadata floor'
PATH="$offline_validation_case/bin:$PATH" TRACE_FILE="$offline_validation_case/override-trace" \
  FAIL_MSRV_METADATA=yes MAKEFLAGS='' MAKEOVERRIDES='' \
  "$make_bin" --no-print-directory -s -C "$offline_validation_case" msrv MSRV=1.92.0
grep -Fxq 'cargo +1.92.0 check --workspace --all-targets --all-features --locked --offline' \
  "$offline_validation_case/override-trace" || fail 'explicit MSRV override was ignored'
for metadata in \
  '{"workspace_members":["fixture"],"packages":[{"id":"fixture"}]}' \
  '{"workspace_members":["a","b"],"packages":[{"id":"a","rust_version":"1.87"},{"id":"b","rust_version":"1.88"}]}' \
  'invalid JSON'; do
  : > "$offline_validation_case/trace"
  if PATH="$offline_validation_case/bin:$PATH" TRACE_FILE="$offline_validation_case/trace" \
    MSRV_FIXTURE_METADATA="$metadata" MAKEFLAGS='' MAKEOVERRIDES='' \
    "$make_bin" --no-print-directory -s -C "$offline_validation_case" msrv > "$offline_validation_case/msrv-refusal.log" 2>&1; then
    fail 'MSRV validation accepted invalid workspace metadata'
  fi
  [[ ! -s "$offline_validation_case/trace" ]] || fail 'invalid MSRV metadata reached compilation'
done
: > "$offline_validation_case/trace"
if PATH="$offline_validation_case/bin:$PATH" TRACE_FILE="$offline_validation_case/trace" \
  FAIL_MSRV_METADATA=yes MAKEFLAGS='' MAKEOVERRIDES='' \
  "$make_bin" --no-print-directory -s -C "$offline_validation_case" msrv > "$offline_validation_case/msrv-cargo-refusal.log" 2>&1; then
  fail 'MSRV validation accepted failed Cargo metadata with valid-looking output'
fi
[[ ! -s "$offline_validation_case/trace" ]] || fail 'failed MSRV discovery reached compilation'

workflow_ci_count="$(grep -Fxc '        run: make ci' "${repo_root}/.github/workflows/ci.yml" || true)"
[[ "${workflow_ci_count}" -eq 1 ]] \
  || fail "hosted CI does not delegate to exactly one complete local gate"
bash "$repo_root/scripts/ci/test-ci-workflow.sh"

schema_version_case="${work_dir}/schema-version"
mkdir -p "${schema_version_case}"
cat > "${schema_version_case}/current.rs" <<'EOF'
pub const REPORT_SCHEMA_VERSION: u32 = 1;
EOF
bash "${repo_root}/scripts/ci/check-schema-versions.sh" "${schema_version_case}" \
  || fail "the schema-version check rejected the current pre-1.0 identifier"
for invalid_schema_version in 0 2 9 10 11 19 20 100; do
  printf 'pub const CACHE_SCHEMA_VERSION: u32 = %s;\n' "${invalid_schema_version}" \
    > "${schema_version_case}/invalid.rs"
  if bash "${repo_root}/scripts/ci/check-schema-versions.sh" "${schema_version_case}" \
    >/dev/null 2>&1; then
    fail "the schema-version check accepted ${invalid_schema_version} before 1.0"
  fi
done

canister_case="${work_dir}/canister-preparation"
mkdir -p "$canister_case/bin"
copy_makefile "$canister_case"
printf 'selected dependency graph\n' > "$canister_case/Cargo.lock"
cp "$canister_case/Cargo.lock" "$canister_case/original-lock"
cat > "$canister_case/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$TRACE_FILE"
case "$1" in
  fetch)
    [[ "$*" == 'fetch --locked' ]] || exit 58
    [[ "${CARGO_NET_OFFLINE:-}" != true ]] || exit 43
    [[ "${CANISTER_FETCH_FAIL:-}" != yes ]] || exit 47
    : > prepared-cache ;;
  clippy)
    [[ -f prepared-cache ]] || exit 59
    case " $* " in *' --locked --offline -- -D warnings '*) ;; *) exit 58 ;; esac
    [[ "${CANISTER_LINT_FAIL:-}" != "$2" ]] || exit 61 ;;
  run)
    [[ -f prepared-cache ]] || exit 59
    case " $* " in *' --locked --offline -- '*) ;; *) exit 58 ;; esac ;;
  *) exit 60 ;;
esac
EOF
chmod +x "$canister_case/bin/cargo"
for selection in 'canister-build build' 'canister-bundle bundle' 'canister-smoke local'; do
  read -r target operation <<< "$selection"
  : > "$canister_case/trace"
  PATH="$canister_case/bin:$PATH" TRACE_FILE="$canister_case/trace" CARGO_NET_OFFLINE=false \
    "$make_bin" --no-print-directory -C "$canister_case" "$target" > "$canister_case/output" 2>&1 \
    || fail "$target did not prepare dependencies before offline runner dispatch"
  printf '%s\n' 'fetch --locked' \
    'clippy -p ic-query --all-targets --no-default-features --locked --offline -- -D warnings' \
    'clippy --workspace --all-targets --all-features --locked --offline -- -D warnings' \
    "run -p ic-query-cli --example governance_smoke --locked --offline -- $operation" > "$canister_case/expected"
  cmp "$canister_case/trace" "$canister_case/expected"
  rm "$canister_case/prepared-cache"
  for refusal in offline network; do
    : > "$canister_case/trace"
    offline=false fetch_fail=no
    if [[ "$refusal" == offline ]]; then offline=true; else fetch_fail=yes; fi
    if PATH="$canister_case/bin:$PATH" TRACE_FILE="$canister_case/trace" \
      CARGO_NET_OFFLINE="$offline" CANISTER_FETCH_FAIL="$fetch_fail" \
      "$make_bin" --no-print-directory -C "$canister_case" "$target" > "$canister_case/output" 2>&1; then
      fail "$target ignored $refusal preparation failure"
    fi
    printf '%s\n' 'fetch --locked' > "$canister_case/expected"
    cmp "$canister_case/trace" "$canister_case/expected"
    [[ ! -e "$canister_case/prepared-cache" ]] || fail 'failed preparation changed cache'
  done
  for lint_scope in -p --workspace; do
    : > "$canister_case/trace"
    if PATH="$canister_case/bin:$PATH" TRACE_FILE="$canister_case/trace" \
      CARGO_NET_OFFLINE=false CANISTER_LINT_FAIL="$lint_scope" \
      "$make_bin" --no-print-directory -j4 -C "$canister_case" "$target" > "$canister_case/output" 2>&1; then
      fail "$target ignored failed Clippy gate"
    fi
    printf '%s\n' 'fetch --locked' \
      'clippy -p ic-query --all-targets --no-default-features --locked --offline -- -D warnings' > "$canister_case/expected"
    if [[ "$lint_scope" == --workspace ]]; then
      printf '%s\n' 'clippy --workspace --all-targets --all-features --locked --offline -- -D warnings' >> "$canister_case/expected"
    fi
    cmp "$canister_case/trace" "$canister_case/expected"
    rm "$canister_case/prepared-cache"
  done
  cmp "$canister_case/Cargo.lock" "$canister_case/original-lock"
done

install_case="${work_dir}/install"
mkdir -p "${install_case}/bin"
cat > "${install_case}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
printf 'cargo %s\n' "$*" > "${TRACE_FILE}"
EOF
chmod +x "${install_case}/bin/cargo"
(
  cd "${repo_root}"
  PATH="${install_case}/bin:${PATH}" TRACE_FILE="${install_case}/trace" \
    "${make_bin}" --no-print-directory install
) >/dev/null
[[ "$(<"${install_case}/trace")" \
  == "cargo install --locked --force --path crates/ic-query-cli --bin icq" ]] \
  || fail "make install does not replace an existing local icq binary"

tools_case="${work_dir}/tools"
ic_arguments='--tool didc --tool ic-wasm --tool icp --tool quill --tool wasm-opt'
mkdir -p "$tools_case/scripts/dev" "$tools_case/ci" "$tools_case/bin"
copy_makefile "$tools_case"
cp "$repo_root/scripts/ci/check-format-tools.sh" "$tools_case/scripts/ci/"
printf 'export SHARED_TOOLING_CARGO_SORT_VERSION=9.8.7\n' > "$tools_case/ci/tool-versions.env"
for family in host ic rust; do
  cat > "$tools_case/scripts/dev/install-$family-tools.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
family="${0##*/install-}"
family="${family%-tools.sh}"
[[ "$PATH" == "$EXPECTED_ROOT/.tools/ic/bin:$EXPECTED_ROOT/.tools/host/bin:$EXPECTED_ROOT/.tools/rust/common/bin:"* ]] || exit 71
printf '%s %s\n' "$family" "$*" >> "$TRACE_FILE"
if [[ "${!#}" == --preflight ]]; then
  [[ "$family" != "${FAIL_PREFLIGHT:-}" ]] || exit 72
  exit 0
fi
if [[ "$family" == rust && "${3:-}" == --package ]]; then
  if [[ "$4" == ic-build ]]; then
    [[ $# == 12 || ( $# == 13 && "${13}" == --check ) ]] || exit 74
    tool=editor
    [[ "$5" == --lockfile && "$6" == Cargo.lock && "$7" == --bin &&
       "$8" == ic-build-release-metadata && "$9" == --profile && "${10}" == debug &&
       "${11}" == --features && "${12}" == cli ]] || exit 74
  else
    [[ $# == 10 || ( $# == 11 && "${11}" == --check ) ]] || exit 74
    tool="${4#cargo-}"
    case "$tool" in audit) version=8.7.6 ;; machete) version=7.6.5 ;; *) exit 74 ;; esac
    [[ "$5" == --version && "$6" == "$version" && "$7" == --bin &&
       "$8" == "$4" && "$9" == --profile && "${10}" == release ]] || exit 74
  fi
  if [[ "$tool" == editor && "${!#}" == --check ]]; then
    [[ "${CARGO_NET_OFFLINE:-}" == true && "${RUSTUP_AUTO_INSTALL:-}" == 0 ]] || exit 74
  fi
  printf '%s/bin/selected %s\n' "$EXPECTED_ROOT" "$tool"
  [[ "$tool" != "${FAIL_FAMILY:-}" ]] || exit 73
  exit 0
fi
[[ "$family" != "${FAIL_FAMILY:-}" ]] || exit 73
EOF
done
cat > "$tools_case/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo %s\n' "$*" >> "$TRACE_FILE"
case "$*" in
  'sort --version') echo cargo-sort 9.8.7 ;;
  'fmt --version') echo rustfmt ;;
  *) exit 74 ;;
esac
EOF
chmod +x "$tools_case/bin/cargo"
for mode in install check; do
  : > "$tools_case/trace"
  if [[ "$mode" == install ]]; then target=install-tools; suffix='';
  else target=tools-check; suffix=' --check'; fi
  PATH="$tools_case/bin:$PATH" TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" \
    MAKEFLAGS='' MAKEOVERRIDES='' IC_TOOL_PINS="$tools_case/ci/ic-tools.tsv" \
    HOST_TOOL_VERSIONS="$tools_case/ci/tool-versions.env" \
    "$make_bin" --no-print-directory -j4 -C "$tools_case" "$target" \
      CARGO_AUDIT_VERSION=8.7.6 CARGO_MACHETE_VERSION=7.6.5 > "$tools_case/$mode.log" 2>&1
  : > "$tools_case/expected"
  if [[ "$mode" == install ]]; then
    printf '%s\n' "ic --consumer $tools_case --pins $tools_case/ci/ic-tools.tsv $ic_arguments --preflight" \
      "rust --consumer $tools_case --versions $tools_case/ci/tool-versions.env --preflight" > "$tools_case/expected"
  fi
  printf '%s\n' "host --consumer $tools_case --versions $tools_case/ci/tool-versions.env$suffix" \
    "rust --consumer $tools_case --versions $tools_case/ci/tool-versions.env$suffix" \
    "ic --consumer $tools_case --pins $tools_case/ci/ic-tools.tsv $ic_arguments$suffix" >> "$tools_case/expected"
  if [[ "$mode" == check ]]; then
    printf '%s\n' "rust --consumer $tools_case --versions $tools_case/ci/tool-versions.env --check" \
      'cargo sort --version' 'cargo fmt --version' >> "$tools_case/expected"
  fi
  printf '%s\n' "rust --consumer $tools_case --package cargo-audit --version 8.7.6 --bin cargo-audit --profile release$suffix" \
    "rust --consumer $tools_case --package cargo-machete --version 7.6.5 --bin cargo-machete --profile release$suffix" \
    "rust --consumer $tools_case --package ic-build --lockfile Cargo.lock --bin ic-build-release-metadata --profile debug --features cli$suffix" >> "$tools_case/expected"
  cmp "$tools_case/expected" "$tools_case/trace" || fail 'tool aggregate changed shared selections or ordering'
  if grep -Ei 'jobserver unavailable|jobserver.*forced' "$tools_case/$mode.log"; then fail 'tool aggregate lost the Make jobserver'; fi
  for family in host ic rust audit machete editor; do
    : > "$tools_case/trace"
    if PATH="$tools_case/bin:$PATH" TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" FAIL_FAMILY="$family" \
      MAKEFLAGS='' MAKEOVERRIDES='' IC_TOOL_PINS="$tools_case/ci/ic-tools.tsv" \
      HOST_TOOL_VERSIONS="$tools_case/ci/tool-versions.env" \
      "$make_bin" --no-print-directory -j4 -C "$tools_case" "$target" \
        CARGO_AUDIT_VERSION=8.7.6 CARGO_MACHETE_VERSION=7.6.5 > "$tools_case/$mode-$family-failure.log" 2>&1; then
      fail "tool aggregate admitted a failed $family selection"
    fi
    awk -v family="$family" '{ print; if ($NF != "--preflight" &&
      (($1 == family && $4 != "--package") || ($4 == "--package" &&
       ($5 == "cargo-" family || (family == "editor" && $5 == "ic-build"))))) exit }' \
      "$tools_case/expected" > "$tools_case/expected-failure"
    cmp "$tools_case/expected-failure" "$tools_case/trace" || fail "tool aggregate continued after $family failure"
  done
done

for family in ic rust; do
  : > "$tools_case/trace"
  if PATH="$tools_case/bin:$PATH" TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" FAIL_PREFLIGHT="$family" \
    MAKEFLAGS='' MAKEOVERRIDES='' IC_TOOL_PINS="$tools_case/ci/ic-tools.tsv" \
    HOST_TOOL_VERSIONS="$tools_case/ci/tool-versions.env" \
    "$make_bin" --no-print-directory -j4 -C "$tools_case" install-tools > "$tools_case/preflight-$family.log" 2>&1; then
    fail "tool setup accepted a failed $family preflight"
  fi
  printf '%s\n' "ic --consumer $tools_case --pins $tools_case/ci/ic-tools.tsv $ic_arguments --preflight" > "$tools_case/expected-preflight"
  if [[ "$family" == rust ]]; then
    printf '%s\n' "rust --consumer $tools_case --versions $tools_case/ci/tool-versions.env --preflight" >> "$tools_case/expected-preflight"
  fi
  cmp "$tools_case/expected-preflight" "$tools_case/trace" || fail 'failed preflight reached installation'
done

cat > "$tools_case/scripts/ci/check-dependencies.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ $# == 2 && "$1" == "$EXPECTED_ROOT/bin/selected audit" && "$2" == "$EXPECTED_ROOT/bin/selected machete" ]] || exit 76
printf 'dependency execution\n' >> "$TRACE_FILE"
EOF
: > "$tools_case/trace"
PATH="$tools_case/bin:$PATH" TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" \
  "$make_bin" --no-print-directory -C "$tools_case" dependency-check \
  CARGO_AUDIT_VERSION=8.7.6 CARGO_MACHETE_VERSION=7.6.5 > "$tools_case/dependency-paths.log" 2>&1
printf '%s\n' "rust --consumer $tools_case --package cargo-audit --version 8.7.6 --bin cargo-audit --profile release --check" \
  "rust --consumer $tools_case --package cargo-machete --version 7.6.5 --bin cargo-machete --profile release --check" \
  'dependency execution' > "$tools_case/expected-dependency"
cmp "$tools_case/expected-dependency" "$tools_case/trace" || fail 'dependency check changed verified executable paths'
for family in audit machete; do
  : > "$tools_case/trace"
  if PATH="$tools_case/bin:$PATH" TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" FAIL_FAMILY="$family" \
    "$make_bin" --no-print-directory -C "$tools_case" dependency-check \
    CARGO_AUDIT_VERSION=8.7.6 CARGO_MACHETE_VERSION=7.6.5 > "$tools_case/dependency-$family-refusal.log" 2>&1; then
    fail 'dependency check accepted a failed selection with valid path stdout'
  fi
  if grep -Fq 'dependency execution' "$tools_case/trace"; then fail 'failed selection dispatched dependency checks'; fi
done
echo 'Dependency setup/check/execution use shared pinned paths and stop on failed admission'

: > "$tools_case/trace"
TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" \
  "$make_bin" --no-print-directory -C "$tools_case" > /dev/null
[[ ! -s "$tools_case/trace" ]] || fail 'default Make goal installed tools'
for target in install-rust-tools rust-tools-check; do
  : > "$tools_case/trace"
  suffix=''
  [[ "$target" != rust-tools-check ]] || suffix=' --check'
  TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" \
    "$make_bin" --no-print-directory -C "$tools_case" "$target" \
    RUST_TOOL_VERSIONS="$tools_case/selected Rust pins.env" > /dev/null
  printf 'rust --consumer %s --versions %s%s\n' "$tools_case" "$tools_case/selected Rust pins.env" "$suffix" > "$tools_case/expected"
  cmp "$tools_case/expected" "$tools_case/trace" || fail 'Rust tool catalog override was lost'
done
: > "$tools_case/trace"
cat > "$tools_case/scripts/dev/cloc.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ "$PATH" == "$EXPECTED_ROOT/.tools/ic/bin:$EXPECTED_ROOT/.tools/host/bin:"* ]] || exit 71
printf 'cloc %s\n' "$*" >> "$TRACE_FILE"
exit "${CLOC_RESULT:-0}"
EOF
TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" \
  "$make_bin" --no-print-directory -C "$tools_case" cloc CLOC_ROOT="$tools_case/selected root" >/dev/null
printf 'cloc %s\n' "$tools_case/selected root" > "$tools_case/expected"
cmp "$tools_case/expected" "$tools_case/trace" || fail 'LOC report lost its selected root'
: > "$tools_case/trace"
TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" \
  "$make_bin" --no-print-directory -C "$tools_case" cloc CLOC_MANIFEST="$tools_case/selected manifest.toml" >/dev/null
printf 'cloc --manifest %s %s\n' "$tools_case/selected manifest.toml" "$tools_case" > "$tools_case/expected"
cmp "$tools_case/expected" "$tools_case/trace" || fail 'LOC report lost its selected manifest'
if TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" CLOC_RESULT=73 \
  "$make_bin" --no-print-directory -C "$tools_case" cloc >/dev/null 2>&1; then
  fail 'Make accepted a failed LOC report'
fi

format_case="$work_dir/format"
mkdir -p "$format_case/bin" "$format_case/ci" "$format_case/scripts/ci" "$format_case/scripts/dev" "$format_case/logs"
copy_makefile "$format_case"
cp "$repo_root/scripts/ci/check-format-tools.sh" "$format_case/scripts/ci/"
printf 'export SHARED_TOOLING_CARGO_SORT_VERSION=9.8.7\n' > "$format_case/ci/tool-versions.env"
cat > "$format_case/scripts/dev/install-rust-tools.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == "--consumer $PWD --versions $PWD/ci/tool-versions.env --check" ]] || exit 1
if [[ "${FORMAT_ADMISSION_STATUS:-0}" != 0 ]]; then echo 'fixture common admission failure' >&2; fi
exit "${FORMAT_ADMISSION_STATUS:-0}"
EOF
cat > "$format_case/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" != install ]]; then
  [[ "$CARGO_NET_OFFLINE" == true && "$RUSTUP_AUTO_INSTALL" == 0 ]] || exit 79
fi
printf '%s\n' "$*" >> "$TRACE_FILE"
case "$*" in
  'sort --version') echo "cargo-sort ${FIXTURE_SORT_VERSION:-9.8.7}" ;;
  'fmt --version') echo rustfmt ;;
esac
if [[ "$*" == "${FAIL_FORMAT_COMMAND:-}" ]]; then
  printf 'formatter stdout: %s\n' "$*"
  printf 'formatter stderr: %s\n' "$*" >&2
  exit 78
fi
EOF
cat > "$format_case/scripts/dev/install-host-tools.sh" <<'EOF'
#!/usr/bin/env bash
[[ "$*" != *--check ]] || exit 0
printf 'host setup %s\n' "$*" >> "$TRACE_FILE"
sleep 0.1
: > host-setup-ready
EOF
chmod +x "$format_case/bin/cargo"
for target in fmt fmt-check; do
  : > "$format_case/trace"
  RUNNER_TEMP="$format_case/logs" PATH="$format_case/bin:$PATH" TRACE_FILE="$format_case/trace" \
    CARGO_NET_OFFLINE=false RUSTUP_AUTO_INSTALL=1 MAKEFLAGS='' MAKEOVERRIDES='' \
    HOST_TOOL_VERSIONS="$format_case/ci/tool-versions.env" \
    "$make_bin" --no-print-directory -C "$format_case" "$target" > "$format_case/success.log" 2>&1
  if [[ "$target" == fmt ]]; then printf 'Formatting... ok\n' > "$format_case/expected-output";
  else printf 'Checking formatting... ok\n' > "$format_case/expected-output"; fi
  cmp "$format_case/expected-output" "$format_case/success.log" || fail 'formatter success output changed'
  if [[ "$target" == fmt ]]; then sort_command='sort --workspace --grouped'; fmt_command='fmt --all';
  else sort_command='sort --workspace --grouped --check'; fmt_command='fmt --all -- --check'; fi
  printf '%s\n' 'sort --version' 'fmt --version' "$sort_command" "$fmt_command" > "$format_case/expected"
  cmp "$format_case/expected" "$format_case/trace" \
    || fail "$target changed formatter order, check flags or the selected setup pin"
  for command in 'sort --version' 'fmt --version' "$sort_command" "$fmt_command"; do
    : > "$format_case/trace"
    if RUNNER_TEMP="$format_case/logs" PATH="$format_case/bin:$PATH" TRACE_FILE="$format_case/trace" FAIL_FORMAT_COMMAND="$command" \
      MAKEFLAGS='' MAKEOVERRIDES='' HOST_TOOL_VERSIONS="$format_case/ci/tool-versions.env" \
      "$make_bin" --no-print-directory -C "$format_case" "$target" > "$format_case/failure.log" 2>&1; then
      fail "$target accepted a failed $command"
    fi
    awk -v stop="$command" '{print; if ($0 == stop) exit}' "$format_case/expected" > "$format_case/expected-failure"
    cmp "$format_case/expected-failure" "$format_case/trace" \
      || fail "$target continued after failed formatter admission or execution"
    if [[ "$command" == "$sort_command" || "$command" == "$fmt_command" ]]; then
      grep -Fq 'FAILED (exit 78)' "$format_case/failure.log" || fail 'formatter failure status was lost'
      retained=false
      for diagnostic in "$format_case/logs/"formatting.*; do
        [[ -f "$diagnostic" ]] || continue
        printf 'Details: %q\n' "$diagnostic" > "$format_case/expected-details"
        grep -Fxf "$format_case/expected-details" "$format_case/failure.log" >/dev/null || continue
        grep -Fxq "formatter stdout: $command" "$diagnostic"
        grep -Fxq "formatter stderr: $command" "$diagnostic"
        retained=true
      done
      [[ "$retained" == true ]] || fail 'formatter failure lost its readable stdout/stderr log'
    fi
  done
done
# Both admission and execution must use the selected executable, even with spaces.
cp "$format_case/bin/cargo" "$format_case/bin/selected cargo"
: > "$format_case/trace"
PATH="$format_case/bin:$PATH" TRACE_FILE="$format_case/trace" MAKEFLAGS='' MAKEOVERRIDES='' \
  HOST_TOOL_VERSIONS="$format_case/ci/tool-versions.env" \
  "$make_bin" --no-print-directory -C "$format_case" fmt-check \
  "FORMAT_CARGO=$format_case/bin/selected cargo" "SHARED_TOOLING_ROOT=$work_dir/external" \
  CARGO_NET_OFFLINE=false RUSTUP_AUTO_INSTALL=1 >/dev/null
cmp "$format_case/expected" "$format_case/trace" || fail 'formatter executable selection diverged'
for target in fmt fmt-check; do
  : > "$format_case/trace"
  if PATH="$format_case/bin:$PATH" TRACE_FILE="$format_case/trace" FIXTURE_SORT_VERSION=9.8.6 \
    MAKEFLAGS='' MAKEOVERRIDES='' HOST_TOOL_VERSIONS="$format_case/ci/tool-versions.env" \
    "$make_bin" --no-print-directory -C "$format_case" "$target" > "$format_case/wrong-pin.log" 2>&1; then
    fail "$target accepted the wrong formatter pin"
  fi
  printf '%s\n' 'sort --version' > "$format_case/expected-admission"
  cmp "$format_case/expected-admission" "$format_case/trace" || fail 'wrong pin dispatched formatting'
  : > "$format_case/trace"
  if PATH="$format_case/bin:$PATH" TRACE_FILE="$format_case/trace" \
    MAKEFLAGS='' MAKEOVERRIDES='' HOST_TOOL_VERSIONS="$format_case/ci/tool-versions.env" \
    "$make_bin" --no-print-directory -C "$format_case" "$target" \
    "FORMAT_CARGO=$format_case/missing cargo" > "$format_case/missing-tool.log" 2>&1; then
    fail "$target accepted an absent formatter"
  fi
  [[ ! -s "$format_case/trace" ]] || fail 'absent formatter dispatched fallback Cargo'
done
: > "$format_case/trace"
if PATH="$format_case/bin:$PATH" TRACE_FILE="$format_case/trace" FORMAT_ADMISSION_STATUS=23 \
  MAKEFLAGS='' MAKEOVERRIDES='' HOST_TOOL_VERSIONS="$format_case/ci/tool-versions.env" \
  "$make_bin" --no-print-directory -C "$format_case" fmt-check > "$format_case/shared-admission-failure.log" 2>&1; then
  fail 'formatter accepted failed common tool admission'
fi
[[ ! -s "$format_case/trace" ]] || fail 'formatter executed after common admission failure'
grep -Fq 'fixture common admission failure' "$format_case/shared-admission-failure.log" || exit 1

perl "${repo_root}/scripts/ci/test-public-docs.pl"
bash "${repo_root}/scripts/ci/test-public-docs.sh"

public_docs_case="${work_dir}/public-docs"
mkdir -p "${public_docs_case}/bin" "${public_docs_case}/tmp"
printf 'retained documentation evidence\n' > "${public_docs_case}/retained-doc"
cat > "${public_docs_case}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  clean) rm -f -- "${DOC_ARTIFACT}" ;;
  doc)
    case " $* " in *' --locked '*) ;; *) exit 58 ;; esac
    case " $* " in *' --offline '*) ;; *) exit 58 ;; esac
    printf 'doc\n' > "$TRACE_FILE"
    # The real parser is covered separately; this fixture checks invocation and
    # failure propagation without invoking rustdoc.
    printf '%s\n' '{"reason":"build-finished","success":true}'
    ;;
  *) exit 2 ;;
esac
EOF
chmod +x "${public_docs_case}/bin/cargo"
# A missing diagnostic set must fail rather than passing by warning count.
if (
  cd "${repo_root}"
  TMPDIR="${public_docs_case}/tmp" PATH="${public_docs_case}/bin:${PATH}" \
    CARGO_TERM_COLOR=always \
    DOC_ARTIFACT="${public_docs_case}/retained-doc" \
    TRACE_FILE="${public_docs_case}/trace" CARGO_NET_OFFLINE=false \
    bash "${repo_root}/scripts/ci/check-public-docs.sh"
) >/dev/null 2>&1; then
  fail "the public documentation check accepted an incomplete diagnostic set"
fi
[[ -f "${public_docs_case}/retained-doc" ]] \
  || fail "the public documentation check erased retained evidence on failure"
[[ -f "${public_docs_case}/trace" ]] \
  || fail "the public documentation check did not select locked/offline Cargo validation"
doc_diagnostics=("${public_docs_case}/tmp/"ic-query-public-docs.*/diagnostics.jsonl)
[[ "${#doc_diagnostics[@]}" == 1 && -s "${doc_diagnostics[0]}" ]] \
  || fail "the failed public documentation check discarded its Cargo diagnostics"

feature_boundary_case="${work_dir}/feature-boundary"
mkdir -p "${feature_boundary_case}/bin" "${feature_boundary_case}/tmp"
cat > "${feature_boundary_case}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
case " $* " in *' --locked '*) ;; *) exit 58 ;; esac
case " $* " in *' --offline '*) ;; *) exit 58 ;; esac
printf '%s\n' "$1" >> "$TRACE_FILE"
if [[ "${FAIL_FEATURE_COMMAND:-}" == "$1" ]]; then
  if [[ "$1" != check || "$*" == *"--features host"* ]]; then
    echo 'fixture Cargo diagnostic' >&2
    exit 51
  fi
fi
if [[ "$1" == tree ]]; then
  package=ic-query
  [[ "$*" != *"--package ic-query-cli"* ]] || package=ic-query-cli
  printf '%s v0.0.0\n' "$package"
  if [[ -n "${FEATURE_TREE_LINE:-}" ]] && \
    [[ "${FEATURE_TREE_SCOPE:-}" == pure && "$*" != *"--features"* \
    || "${FEATURE_TREE_SCOPE:-}" == host && "$*" == *"--features host"* \
    || "${FEATURE_TREE_SCOPE:-}" == canister && "$*" == *"--features canister"* \
    || "${FEATURE_TREE_SCOPE:-}" == cli && "$package" == ic-query-cli ]]; then
    case "$FEATURE_TREE_LINE" in
      */*)
        [[ "$*" != *"normal,build,features"* ]] || \
          printf '%s feature "%s"\n' "${FEATURE_TREE_LINE%%/*}" "${FEATURE_TREE_LINE#*/}"
        ;;
      *) printf '%s v0.0.0\n' "$FEATURE_TREE_LINE" ;;
    esac
  fi
fi
EOF
chmod +x "${feature_boundary_case}/bin/cargo"
TMPDIR="${feature_boundary_case}/tmp" PATH="${feature_boundary_case}/bin:${PATH}" \
  TRACE_FILE="${feature_boundary_case}/trace" CARGO_NET_OFFLINE=false \
  bash "${repo_root}/scripts/ci/check-library-feature-boundaries.sh" >/dev/null \
  || fail "the feature-boundary check rejected successful Cargo commands"
[[ -z "$(find "${feature_boundary_case}/tmp" -mindepth 1 -print -quit)" ]] \
  || fail "the successful feature-boundary check left temporary files"
for failed_command in check test tree; do
  mkdir "${feature_boundary_case}/tmp/$failed_command"
  : > "${feature_boundary_case}/trace"
  if TMPDIR="${feature_boundary_case}/tmp/$failed_command" PATH="${feature_boundary_case}/bin:${PATH}" \
    FAIL_FEATURE_COMMAND="$failed_command" TRACE_FILE="${feature_boundary_case}/trace" \
    CARGO_NET_OFFLINE=false \
    bash "${repo_root}/scripts/ci/check-library-feature-boundaries.sh" \
      >"${feature_boundary_case}/failure.log" 2>&1; then
    feature_boundary_status=0
  else
    feature_boundary_status="$?"
  fi
  [[ "$feature_boundary_status" == 51 ]] \
    || fail "the feature-boundary check lost the failed $failed_command status"
  [[ "$(tail -n 1 "${feature_boundary_case}/trace")" == "$failed_command" ]] \
    || fail "the feature-boundary check continued after failed $failed_command"
  grep -Fq 'fixture Cargo diagnostic' "${feature_boundary_case}/failure.log" \
    || fail "the feature-boundary check discarded the Cargo diagnostic"
  feature_diagnostics=("${feature_boundary_case}/tmp/$failed_command/"ic-query-feature-boundary.*)
  [[ "${#feature_diagnostics[@]}" == 1 && -d "${feature_diagnostics[0]}" ]] \
    || fail "the failed feature-boundary check discarded its diagnostics directory"
  if [[ "$failed_command" == check ]]; then
    grep -Fq 'fixture Cargo diagnostic' "${feature_diagnostics[0]}"/check.* \
      || fail "the failed compilation discarded its retained Cargo log"
  fi
done

for dependency in ic-host-artifacts ic-host-fs ic-host-process ic-host-tools candid_parser \
  ic-host-tools/candid ic-host-tools/ic-interface ic-host-artifacts/archive ic-host-artifacts/gzip \
  ic-host-artifacts/wasm; do
  scope=host
  case "$dependency" in
    ic-host-artifacts|ic-host-fs|ic-host-tools) scope=pure ;;
  esac
  : > "${feature_boundary_case}/trace"
  if TMPDIR="${feature_boundary_case}/tmp" PATH="${feature_boundary_case}/bin:${PATH}" \
    FEATURE_TREE_SCOPE="$scope" FEATURE_TREE_LINE="$dependency" \
    TRACE_FILE="${feature_boundary_case}/trace" \
    bash "${repo_root}/scripts/ci/check-library-feature-boundaries.sh" \
      >"${feature_boundary_case}/dependency-failure.log" 2>&1; then
    fail "the $scope feature boundary accepted $dependency"
  fi
  case "$dependency" in
    */*) diagnostic="forbidden dependency feature edge: $dependency" ;;
    *) diagnostic="forbidden $dependency:" ;;
  esac
  grep -Fq "$diagnostic" "${feature_boundary_case}/dependency-failure.log" \
    || fail "the $scope feature boundary lost the forbidden dependency diagnostic"
  if grep -Eq '^(check|test)$' "${feature_boundary_case}/trace"; then
    fail "the $scope feature boundary compiled or tested before admitting its dependency graph"
  fi
  grep -Fq 'error: dependency graph check failed:' "${feature_boundary_case}/dependency-failure.log" \
    || fail "the $scope feature boundary lost its failing graph context"
done

for scope in canister cli; do
  for dependency in ic-build ic-host-process ic-testkit wat; do
    if TMPDIR="${feature_boundary_case}/tmp" PATH="${feature_boundary_case}/bin:${PATH}" \
      FEATURE_TREE_SCOPE="$scope" FEATURE_TREE_LINE="$dependency" \
      TRACE_FILE="${feature_boundary_case}/trace" \
      bash "${repo_root}/scripts/ci/check-library-feature-boundaries.sh" \
        >"${feature_boundary_case}/tooling-refusal.log" 2>&1; then
      fail "the $scope production graph accepted $dependency"
    fi
    grep -Fq "forbidden $dependency:" "${feature_boundary_case}/tooling-refusal.log" \
      || fail "the $scope production graph lost the tooling diagnostic"
  done
done

for dependency in ic-host-fs ic-platform ic-host-tools/certificate; do
  TMPDIR="${feature_boundary_case}/tmp" PATH="${feature_boundary_case}/bin:${PATH}" \
    FEATURE_TREE_SCOPE=host FEATURE_TREE_LINE="$dependency" \
    TRACE_FILE="${feature_boundary_case}/trace" \
    bash "${repo_root}/scripts/ci/check-library-feature-boundaries.sh" \
      >"${feature_boundary_case}/dependency-accepted.log" 2>&1 \
      || fail "the host feature boundary rejected its current dependency $dependency"
done

retention_case="$work_dir/fixture-retention"
mkdir -p "$retention_case/bin"
export RETENTION_REAL_CP
RETENTION_REAL_CP="$(command -v cp)"
cat > "$retention_case/bin/cp" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
reject() { echo 'fixture copy diagnostic' >&2; exit 23; }
case "$PWD" in "$RETENTION_TMP"/ic-query-*) reject ;; esac
for argument in "$@"; do
  case "$argument" in "$RETENTION_TMP"/ic-query-*) reject ;; esac
done
exec "$RETENTION_REAL_CP" "$@"
EOF
chmod +x "$retention_case/bin/cp"
for fixture in release-metadata publish-guards release-guards; do
  mkdir "$retention_case/$fixture"
  printf 'caller evidence\n' > "$retention_case/$fixture/evidence"
  status=0
  TMPDIR="$retention_case/$fixture" RETENTION_TMP="$retention_case/$fixture" \
    PATH="$retention_case/bin:$PATH" bash "$repo_root/scripts/ci/check-$fixture.sh" \
    > "$retention_case/$fixture.log" 2>&1 || status=$?
  [[ "$status" == 23 ]] || fail "$fixture lost the failed fixture setup status"
  retained=("$retention_case/$fixture/"ic-query-*)
  for directory in "${retained[@]}"; do
    [[ -d "$directory" ]] || fail "$fixture discarded its failed setup"
    grep -Fq "retained: $directory" "$retention_case/$fixture.log" \
      || fail "$fixture did not identify its retained evidence"
  done
  grep -Fxq 'caller evidence' "$retention_case/$fixture/evidence" \
    || fail "$fixture changed caller-owned evidence"
done

dependency_check_case="${work_dir}/dependency-check"
mkdir -p "${dependency_check_case}/bin" "${dependency_check_case}/tmp"
cat > "${dependency_check_case}/bin/git" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ "${GIT_TERMINAL_PROMPT:-}" == 0 ]] || exit 67
if [[ "$1" == -C ]]; then
  printf '1111111111111111111111111111111111111111\n'
  exit 0
fi
advisory_db="${!#}"
[[ "${advisory_db}" == "${EXPECTED_TMP_ROOT}"/ic-query-dependency-check.*/rustsec/db ]] || exit 69
[[ ! -e "${advisory_db}" ]] || exit 70
mkdir -p "${advisory_db}"
printf 'fetch\n' >> "${TRACE_FILE}"
echo 'fixture clone diagnostic' >&2
[[ -z "${FAIL_FETCH:-}" ]] || exit 53
EOF
chmod +x "${dependency_check_case}/bin/git"
cat > "${dependency_check_case}/bin/selected audit" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
  audit)
    shift
    [[ "${1:-}" == "--no-fetch" ]] || exit 60
    shift
    [[ "${1:-}" == "--db" ]] || exit 61
    advisory_db="${2:-}"
    shift 2
    [[ "${advisory_db}" == "${EXPECTED_TMP_ROOT}"/ic-query-dependency-check.*/rustsec/db ]] \
      || exit 62
    [[ -d "${advisory_db}" ]] || exit 63
    [[ "$(cat "${advisory_db}/../revision")" == 1111111111111111111111111111111111111111 ]] || exit 63
    [[ "$*" == "--deny warnings --ignore RUSTSEC-2021-0127 --ignore RUSTSEC-2024-0436" ]] \
      || exit 64
    printf 'audit\n' >> "${TRACE_FILE}"
    [[ -z "${FAIL_AUDIT:-}" ]] || exit 52
    ;;
  *)
    exit 66
    ;;
esac
EOF
cat > "${dependency_check_case}/bin/selected machete" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ "${CARGO+x}" != x && "${CARGO_NET_OFFLINE:-}" == true && $# == 1 && "$1" == --with-metadata ]] || exit 65
printf 'machete\n' >> "${TRACE_FILE}"
[[ -z "${FAIL_MACHETE:-}" ]] || exit 57
EOF
chmod +x "${dependency_check_case}/bin/selected audit" "${dependency_check_case}/bin/selected machete"
TMPDIR="${dependency_check_case}/tmp" PATH="${dependency_check_case}/bin:${PATH}" \
  EXPECTED_TMP_ROOT="${dependency_check_case}/tmp" \
  TRACE_FILE="${dependency_check_case}/trace" CARGO=fixture-cargo \
  bash "${repo_root}/scripts/ci/check-dependencies.sh" \
    "${dependency_check_case}/bin/selected audit" "${dependency_check_case}/bin/selected machete" >/dev/null
printf '%s\n' fetch audit machete > "${dependency_check_case}/expected-trace"
cmp -s "${dependency_check_case}/expected-trace" "${dependency_check_case}/trace" \
  || fail "the dependency check did not run one isolated audit before cargo machete"
[[ -z "$(find "${dependency_check_case}/tmp" -mindepth 1 -print -quit)" ]] \
  || fail "the successful dependency check left its advisory database behind"

: > "${dependency_check_case}/trace"
if TMPDIR="${dependency_check_case}/tmp" PATH="${dependency_check_case}/bin:${PATH}" \
  EXPECTED_TMP_ROOT="${dependency_check_case}/tmp" \
  TRACE_FILE="${dependency_check_case}/trace" FAIL_AUDIT=1 \
  bash "${repo_root}/scripts/ci/check-dependencies.sh" \
    "${dependency_check_case}/bin/selected audit" "${dependency_check_case}/bin/selected machete" >/dev/null 2>&1; then
  dependency_check_status=0
else
  dependency_check_status="$?"
fi
[[ "${dependency_check_status}" -eq 52 ]] \
  || fail "the dependency check hid a failed cargo audit"
printf '%s\n' fetch audit > "${dependency_check_case}/expected-trace"
cmp -s "${dependency_check_case}/expected-trace" "${dependency_check_case}/trace" \
  || fail "the dependency check continued after a failed cargo audit"
audit_evidence=("${dependency_check_case}/tmp/"ic-query-dependency-check.*/rustsec/revision)
[[ "${#audit_evidence[@]}" == 1 && -f "${audit_evidence[0]}" ]] \
  || fail "the failed audit discarded its selected database identity"

: > "${dependency_check_case}/trace"
dependency_check_status=0
TMPDIR="${dependency_check_case}/tmp" PATH="${dependency_check_case}/bin:${PATH}" \
  EXPECTED_TMP_ROOT="${dependency_check_case}/tmp" \
  TRACE_FILE="${dependency_check_case}/trace" FAIL_MACHETE=1 \
  bash "${repo_root}/scripts/ci/check-dependencies.sh" \
    "${dependency_check_case}/bin/selected audit" "${dependency_check_case}/bin/selected machete" \
    > "${dependency_check_case}/machete-failure.log" 2>&1 || dependency_check_status=$?
[[ "$dependency_check_status" == 57 ]] || fail 'dependency check hid the selected Machete failure'
printf '%s\n' fetch audit machete > "${dependency_check_case}/expected-trace"
cmp "${dependency_check_case}/expected-trace" "${dependency_check_case}/trace" \
  || fail 'dependency check changed Audit/Machete execution order'
audit_evidence=("${dependency_check_case}/tmp/"ic-query-dependency-check.*/rustsec/revision)
[[ "${#audit_evidence[@]}" == 2 ]] || fail 'failed Machete discarded its database identity'

: > "${dependency_check_case}/trace"
if TMPDIR="${dependency_check_case}/tmp" PATH="${dependency_check_case}/bin:${PATH}" \
  EXPECTED_TMP_ROOT="${dependency_check_case}/tmp" \
  TRACE_FILE="${dependency_check_case}/trace" FAIL_FETCH=1 \
  bash "${repo_root}/scripts/ci/check-dependencies.sh" \
    "${dependency_check_case}/bin/selected audit" "${dependency_check_case}/bin/selected machete" >/dev/null 2>&1; then
  dependency_check_status=0
else
  dependency_check_status="$?"
fi
[[ "${dependency_check_status}" -ne 0 ]] \
  || fail "the dependency check hid a failed database fetch"
printf '%s\n' fetch > "${dependency_check_case}/expected-trace"
cmp -s "${dependency_check_case}/expected-trace" "${dependency_check_case}/trace" \
  || fail "the dependency check continued after a failed database fetch"
preparation_logs=("${dependency_check_case}/tmp/"ic-query-dependency-check.*/rustsec/prepare.log)
[[ "${#preparation_logs[@]}" == 3 ]] || fail "the failed fetch discarded its preparation log"
for log in "${preparation_logs[@]}"; do
  grep -Fq 'fixture clone diagnostic' "$log" || fail "the retained preparation log lost its diagnostic"
done

package_contents_case="${work_dir}/package-contents"
mkdir -p "${package_contents_case}/bin"
cat > "${package_contents_case}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
exit 43
EOF
chmod +x "${package_contents_case}/bin/cargo"
if PATH="${package_contents_case}/bin:${PATH}" \
  bash "${repo_root}/scripts/ci/check-package-contents.sh" >/dev/null 2>&1; then
  package_contents_status=0
else
  package_contents_status="$?"
fi
[[ "${package_contents_status}" -eq 43 ]] \
  || fail "the package contents check hid a Cargo listing failure"

package_workspace_case="${work_dir}/package-workspace"
mkdir -p "${package_workspace_case}/bin"
cat > "${package_workspace_case}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
printf 'cargo %s\n' "$*" >> "${TRACE_FILE}"
[[ "$*" != *"-p ${FAIL_PACKAGE:-unset} "* ]] || exit 42
EOF
chmod +x "${package_workspace_case}/bin/cargo"
PATH="${package_workspace_case}/bin:${PATH}" \
  TRACE_FILE="${package_workspace_case}/trace" \
  bash "${repo_root}/scripts/ci/package-workspace.sh" --offline >/dev/null
expected_cli_package='cargo package -p ic-query-cli --locked --config patch.crates-io.ic-query.path="crates/ic-query" --offline'
printf '%s\n' 'cargo package -p ic-query --locked --offline' "${expected_cli_package}" \
  > "${package_workspace_case}/expected-trace"
cmp -s "${package_workspace_case}/expected-trace" "${package_workspace_case}/trace" \
  || fail "the workspace package check did not package the library then verify the CLI against it"

for failed_package in ic-query ic-query-cli; do
  : > "${package_workspace_case}/trace"
  if PATH="${package_workspace_case}/bin:${PATH}" \
    TRACE_FILE="${package_workspace_case}/trace" FAIL_PACKAGE="$failed_package" \
    bash "${repo_root}/scripts/ci/package-workspace.sh" --offline >/dev/null 2>&1; then
    package_status=0
  else
    package_status="$?"
  fi
  [[ "$package_status" == 42 ]] || fail "workspace packaging lost Cargo failure status"
  printf '%s\n' 'cargo package -p ic-query --locked --offline' > "${package_workspace_case}/expected-trace"
  if [[ "$failed_package" == ic-query-cli ]]; then
    printf '%s\n' "$expected_cli_package" >> "${package_workspace_case}/expected-trace"
  fi
  cmp -s "${package_workspace_case}/expected-trace" "${package_workspace_case}/trace" \
    || fail "workspace packaging repeated a command or continued after failure"
done
fixture_complete=true
