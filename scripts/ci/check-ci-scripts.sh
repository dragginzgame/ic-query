#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
make_bin="$(command -v make)"
export CI_FIXTURE_REAL_MAKE="$make_bin"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-ci-scripts.XXXXXX")"
trap 'if [[ $? == 0 ]]; then rm -rf -- "$work_dir"; else echo "CI script fixtures retained: $work_dir" >&2; fi' EXIT

fail() {
  echo "error: $*" >&2
  exit 1
}

copy_makefile() {
  mkdir -p "$1/make" "$1/scripts/ci"
  cp "$repo_root/Makefile" "$1/Makefile"
  cp "$repo_root/make/"{tools,release,rust-format,execution}.mk "$1/make/"
  cp "$repo_root/scripts/ci/"{check-release-source,check-make-execution,run-formatting}.sh "$1/scripts/ci/"
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

.PHONY: execution-validate execution-probe execution-child evidence-success evidence-failure
execution-validate:
	+@VALIDATION_REPOSITORY_ROOT="$(CURDIR)" bash "$(LOGGER)" execution-probe
execution-probe:
	@printf '%s\n' '$(RELEASE_REMOTE)' >> "$${TRACE_FILE}"
	+$(MAKE) --no-print-directory execution-child
execution-child:
	@printf '%s\n' '$(RELEASE_REMOTE)' >> "$${TRACE_FILE}"
evidence-success:
	@echo retained-success-marker
evidence-failure:
	@echo 'error: retained-failure-marker'
	@exit 7
MAKE
  for mode in -i -n -t -q -kin --ignore-errors --dry-run --just-print --recon --touch --question; do
    status=0
    env -u MAKEFLAGS -u MFLAGS -u MAKEOVERRIDES -u GNUMAKEFLAGS \
      TRACE_FILE="$fixture/trace" "$make_bin" --no-print-directory -C "$fixture" \
      "$mode" execution-probe > "$fixture/blocked.log" 2>&1 || status=$?
    [[ "$status" == 2 && ! -e "$fixture/trace" ]] || fail "Make admitted execution mode $mode"
  done
  for mode in i n t q '--no-print-directory -i'; do
    status=0
    env -u MAKEFLAGS -u MFLAGS -u MAKEOVERRIDES -u GNUMAKEFLAGS \
      MAKEFLAGS="$mode" TRACE_FILE="$fixture/trace" "$make_bin" --no-print-directory \
      -C "$fixture" execution-probe > "$fixture/blocked.log" 2>&1 || status=$?
    [[ "$status" == 2 && ! -e "$fixture/trace" ]] || fail "Make admitted inherited MAKEFLAGS=$mode"
  done
  for mode in -i -n -t -q -kin --ignore-errors --dry-run --touch --question; do
    for flags in '' --no-print-directory; do
      status=0
      env -u MAKEFLAGS -u MFLAGS -u MAKEOVERRIDES -u GNUMAKEFLAGS \
        TRACE_FILE="$fixture/trace" "$make_bin" --no-print-directory -C "$fixture" \
        "$mode" "MAKEFLAGS=$flags" execution-probe > "$fixture/blocked.log" 2>&1 || status=$?
      [[ "$status" == 2 && ! -e "$fixture/trace" ]] || fail "MAKEFLAGS override hid invocation mode $mode"
    done
  done
  for flags in '' --no-print-directory; do
    status=0
    env -u MAKEFLAGS -u MFLAGS -u MAKEOVERRIDES -u GNUMAKEFLAGS \
      TRACE_FILE="$fixture/trace" "$make_bin" --no-print-directory -C "$fixture" \
      "MFLAGS=$flags" execution-probe > "$fixture/blocked.log" 2>&1 || status=$?
    [[ "$status" == 2 && ! -e "$fixture/trace" ]] || fail 'Make admitted replaced invocation evidence'
  done
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
  local selected_directory prior_directory record_target record_result target seconds log
  for target in evidence-success evidence-failure; do
    status=0
    VALIDATION_REPOSITORY_ROOT="$fixture" VALIDATION_LOG_DIR="$fixture/logs" \
      VALIDATION_FAILURE_LOG_DIR="$fixture/failures" \
      bash "$repo_root/scripts/ci/run-validation-targets.sh" "$target" \
      > "$fixture/evidence-output" 2>&1 || status=$?
    selected_directory="$(sed -n 's/^Validation logs and timings: //p' "$fixture/evidence-output")"
    IFS=$'\t' read -r record_target record_result seconds log < <(sed -n '2p' "$selected_directory/timings.tsv")
    [[ "$record_target" == "$target" && "$seconds" =~ ^[0-9]+$ && -f "$log" ]] \
      || fail 'selected validation evidence lost target, timing or raw log'
    if [[ "$target" == evidence-success ]]; then
      [[ "$status" == 0 && "$record_result" == PASS ]] || fail 'passing validation lost its timing result'
      grep -Fxq retained-success-marker "$log" || fail 'passing validation lost its raw output'
      prior_directory="$selected_directory"
      cp "$log" "$fixture/previous-success.log"
    else
      [[ "$status" == 2 && "$record_result" == FAIL ]] || fail 'validation lost Make failure status'
      grep -Fq retained-failure-marker "$log" || fail 'failed validation lost its raw output'
      cmp "$fixture/previous-success.log" "$prior_directory/0.log" \
        || fail 'later validation changed earlier success evidence'
    fi
  done
  mkdir -p "$fixture/failures"
  printf 'retained prior validation\n' > "$fixture/failures/latest.log"
  cp "$fixture/failures/latest.log" "$fixture/previous.log"
  for mode in i n t q v --ignore-errors --dry-run --touch --question --version; do
    status=0
    env -u MAKEFLAGS -u MFLAGS -u MAKEOVERRIDES -u GNUMAKEFLAGS MAKEFLAGS="$mode" \
      VALIDATION_REPOSITORY_ROOT="$fixture" VALIDATION_FAILURE_LOG_DIR="$fixture/failures" \
      TRACE_FILE="$fixture/trace" bash "$repo_root/scripts/ci/run-validation-targets.sh" \
      --fail-fast execution-probe > "$fixture/validation-$mode.log" 2>&1 || status=$?
    [[ "$status" != 0 && ! -e "$fixture/trace" ]] \
      || fail "Validation admitted inherited mode $mode"
    grep -Fq 'requires recipe execution and failure propagation' "$fixture/validation-$mode.log" \
      || fail "Validation lost its execution admission diagnostic for $mode"
    cmp "$fixture/previous.log" "$fixture/failures/latest.log" \
      || fail 'Execution admission changed prior validation evidence'
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
  host-tools-check
  format-tools-check
  changelog-check
  dependency-pins-check
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
  clippy
  test
  package
)
for target in "${expected_ci_targets[@]}"; do
  printf 'make --no-print-directory -C %s -- %s\n' "${repo_root}" "${target}"
done > "${ci_gate_case}/expected-trace"
cmp -s "${ci_gate_case}/expected-trace" "${ci_gate_case}/trace" \
  || fail "make ci ran an unexpected target sequence"

for failed_target in format-tools-check changelog-check dependency-pins-check doc-links-check test; do
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
mkdir -p "$tools_case/scripts/dev"
copy_makefile "$tools_case"
for family in host ic rust; do
  cat > "$tools_case/scripts/dev/install-$family-tools.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
family="${0##*/install-}"
family="${family%-tools.sh}"
[[ "$PATH" == "$EXPECTED_ROOT/.tools/host/bin:$EXPECTED_ROOT/.tools/ic/bin:"* ]] || exit 71
printf '%s %s\n' "$family" "$*" >> "$TRACE_FILE"
[[ "$family" != "${FAIL_FAMILY:-}" ]] || exit 73
EOF
done
for mode in install check; do
  : > "$tools_case/trace"
  if [[ "$mode" == install ]]; then target=install-tools; suffix='';
  else target=tools-check; suffix=' --check'; fi
  TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" \
    MAKEFLAGS='' MAKEOVERRIDES='' IC_TOOL_PINS="$tools_case/ci/ic-tools.tsv" \
    HOST_TOOL_VERSIONS="$tools_case/ci/tool-versions.env" \
    "$make_bin" --no-print-directory -C "$tools_case" "$target" >/dev/null
  printf '%s\n' "host --consumer $tools_case --versions $tools_case/ci/tool-versions.env --with-ripgrep --with-cloc$suffix" \
    "ic --consumer $tools_case --pins $tools_case/ci/ic-tools.tsv$suffix" > "$tools_case/expected"
  cmp -s "$tools_case/expected" "$tools_case/trace" \
    || fail "local tool setup/check changed ordering, pins or offline selection"
  : > "$tools_case/trace"
  if TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" FAIL_FAMILY=host \
    MAKEFLAGS='' MAKEOVERRIDES='' IC_TOOL_PINS="$tools_case/ci/ic-tools.tsv" \
    HOST_TOOL_VERSIONS="$tools_case/ci/tool-versions.env" \
    "$make_bin" --no-print-directory -C "$tools_case" "$target" >/dev/null 2>&1; then
    fail "tool setup/check accepted a host failure"
  fi
  printf '%s\n' "host --consumer $tools_case --versions $tools_case/ci/tool-versions.env --with-ripgrep --with-cloc$suffix" > "$tools_case/expected"
  cmp -s "$tools_case/expected" "$tools_case/trace" \
    || fail "tool setup/check continued after a failed host command"
done

: > "$tools_case/trace"
TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" \
  "$make_bin" --no-print-directory -C "$tools_case" >/dev/null
[[ ! -s "$tools_case/trace" ]] || fail 'default Make goal installed tools'
for target in install-rust-tools rust-tools-check; do
  : > "$tools_case/trace"
  suffix=''
  [[ "$target" != rust-tools-check ]] || suffix=' --check'
  TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" \
    "$make_bin" --no-print-directory -C "$tools_case" "$target" \
    RUST_TOOL_VERSIONS="$tools_case/selected Rust pins.env" >/dev/null
  printf 'rust --consumer %s --versions %s%s\n' "$tools_case" \
    "$tools_case/selected Rust pins.env" "$suffix" > "$tools_case/expected"
  cmp "$tools_case/expected" "$tools_case/trace" \
    || fail 'optional Rust setup/check lost its selected catalog or offline mode'
  if TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" FAIL_FAMILY=rust \
    "$make_bin" --no-print-directory -C "$tools_case" "$target" >/dev/null 2>&1; then
    fail 'optional Rust setup/check accepted installer failure'
  fi
done
: > "$tools_case/trace"
cat > "$tools_case/scripts/dev/cloc.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ "$PATH" == "$EXPECTED_ROOT/.tools/host/bin:$EXPECTED_ROOT/.tools/ic/bin:"* ]] || exit 71
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
[[ "$*" == "--consumer $PWD --package cargo-sort --version 9.8.7 --bin cargo-sort --profile release --check" ]]
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
  if [[ "$target" == fmt ]]; then sort_command='sort --workspace'; fmt_command='fmt --all';
  else sort_command='sort --workspace --check'; fmt_command='fmt --all -- --check'; fi
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
cp "$repo_root/scripts/dev/install-rust-tools.sh" "$format_case/scripts/dev/"
cp "$repo_root/scripts/ci/verify-file-checksum.sh" "$format_case/scripts/ci/"
cp "$repo_root/scripts/ci/run-validation-targets.sh" "$format_case/scripts/ci/"
printf '#!/usr/bin/env bash\nexit 0\n' > "$format_case/scripts/ci/verify-shared-tooling-snapshot.sh"
mkdir -p "$format_case/.tools/rust/bin"
printf '#!/usr/bin/env bash\necho cargo-sort 9.8.6\n' > "$format_case/.tools/rust/bin/cargo-sort"
printf '#!/usr/bin/env bash\necho cargo-sort 9.8.5\n' > "$format_case/bin/cargo-sort"
chmod +x "$format_case/.tools/rust/bin/cargo-sort" "$format_case/bin/cargo-sort"
cp "$format_case/.tools/rust/bin/cargo-sort" "$format_case/local-before"
cp "$format_case/bin/cargo-sort" "$format_case/global-before"
printf '#!/usr/bin/env bash\necho "host: fixture-host"\n' > "$format_case/bin/rustc"
chmod +x "$format_case/bin/rustc"
cat > "$format_case/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
  install)
    [[ -f host-setup-ready ]]
    if [[ "$*" == 'install --locked cargo-sort --version 9.8.7' ]]; then
      # Reproduce the old global setup while leaving the stale local winner.
      printf '#!/usr/bin/env bash\necho cargo-sort 9.8.7\n' > "$FIXTURE_GLOBAL_SORT"
      exit 0
    fi
    if [[ "$2" != cargo-sort ]]; then
      printf '%s\n' "$*" >> "$TRACE_FILE"
      [[ "$*" == 'install --locked cargo-audit --version 8.7.6' ||
         "$*" == 'install --locked cargo-machete --version 7.6.5' ]]
      exit 0
    fi
    [[ $# == 13 && "$3" == --version && "$4" == =9.8.7 && "$5" == --locked &&
       "$6" == --root && "$8" == --target-dir && "$9" == "$7/build" &&
       "${10}" == --bin && "${11}" == cargo-sort && "${12}" == --registry &&
       "${13}" == crates-io && "$RUSTUP_AUTO_INSTALL" == 0 ]]
    printf 'formatter setup 9.8.7\n' >> "$TRACE_FILE"
    [[ "${FIXTURE_SORT_INSTALL_FAIL:-}" != yes ]] || exit 23
    mkdir -p "$7/bin"
    cat > "$7/bin/cargo-sort" <<'SORT'
#!/usr/bin/env bash
set -euo pipefail
[[ "$CARGO_NET_OFFLINE" == true && "$RUSTUP_AUTO_INSTALL" == 0 ]]
printf 'sort %s\n' "$*" >> "$TRACE_FILE"
if [[ "$*" == --version ]]; then echo 'cargo-sort 9.8.7';
else [[ "$*" == '--workspace --check' ]]; fi
SORT
    chmod +x "$7/bin/cargo-sort"
    jq -n '{installs:{"cargo-sort 9.8.7 (registry+https://github.com/rust-lang/crates.io-index)":
      {version_req:"=9.8.7",bins:["cargo-sort"],profile:"release",target:"fixture-host",rustc:"fixture rustc"}}}' > "$7/.crates2.json"
    ;;
  sort) shift; exec cargo-sort "$@" ;;
  fmt)
    [[ "$CARGO_NET_OFFLINE" == true && "$RUSTUP_AUTO_INSTALL" == 0 ]]
    printf '%s\n' "$*" >> "$TRACE_FILE"
    if [[ "$*" == 'fmt --version' ]]; then echo rustfmt;
    else [[ "$*" == 'fmt --all -- --check' ]]; fi
    ;;
  *) exit 80 ;;
esac
CARGO
run_formatter_make() {
  PATH="$format_case/bin:$PATH" TRACE_FILE="$format_case/trace" \
    FIXTURE_GLOBAL_SORT="$format_case/bin/cargo-sort" \
    MAKEFLAGS='' MAKEOVERRIDES='' HOST_TOOL_VERSIONS="$format_case/ci/tool-versions.env" \
    "$make_bin" --no-print-directory -C "$format_case" "$@"
}
if run_formatter_make fmt-check > "$format_case/stale.log" 2>&1; then
  fail 'formatting admitted the stale local formatter'
fi
: > "$format_case/trace"
if run_formatter_make -j4 ci > "$format_case/early-refusal.log" 2>&1; then
  fail 'CI admitted a missing selected formatter'
fi
[[ ! -s "$format_case/trace" ]] || fail 'missing formatter dispatched Cargo before early CI refusal'
grep -Fq 'missing selected Cargo tool: package=cargo-sort version=9.8.7 target=bin:cargo-sort profile=release' "$format_case/early-refusal.log"
grep -Fq 'make install-format-tools' "$format_case/early-refusal.log"
: > "$format_case/trace"
run_formatter_make -j4 install-dev CARGO_AUDIT_VERSION=8.7.6 CARGO_MACHETE_VERSION=7.6.5 \
  > "$format_case/setup.log" 2>&1
run_formatter_make fmt-check > "$format_case/prepared.log" 2>&1 \
  || fail 'install-dev did not prepare the formatter selected by fmt-check'
printf '%s\n' "host setup --consumer $format_case --versions $format_case/ci/tool-versions.env --with-ripgrep --with-cloc" \
  'formatter setup 9.8.7' 'install --locked cargo-audit --version 8.7.6' \
  'install --locked cargo-machete --version 7.6.5' 'sort --version' 'fmt --version' \
  'sort --workspace --check' 'fmt --all -- --check' > "$format_case/expected"
cmp "$format_case/expected" "$format_case/trace" || fail 'formatter setup or offline lookup changed'
cmp "$format_case/local-before" "$format_case/.tools/rust/bin/cargo-sort"
cmp "$format_case/global-before" "$format_case/bin/cargo-sort"
cp "$format_case/ci/tool-versions.env" "$format_case/selected-pin"
printf 'export SHARED_TOOLING_CARGO_SORT_VERSION=9.8.8\n' > "$format_case/ci/tool-versions.env"
: > "$format_case/trace"
if run_formatter_make -j4 fmt-check > "$format_case/changed-selection.log" 2>&1; then
  fail 'formatter admission accepted a different installation selection'
fi
[[ ! -s "$format_case/trace" ]] || fail 'changed formatter selection dispatched a fallback'
cp "$format_case/selected-pin" "$format_case/ci/tool-versions.env"
slot="$format_case/.tools/rust/cargo-sort-9.8.7-bin-cargo-sort-release"
mv "$slot/installed" "$format_case/prepared-formatter"
: > "$format_case/trace"
if FIXTURE_SORT_INSTALL_FAIL=yes run_formatter_make -j4 install-dev \
  CARGO_AUDIT_VERSION=8.7.6 CARGO_MACHETE_VERSION=7.6.5 > "$format_case/setup-failure.log" 2>&1; then
  fail 'install-dev admitted a failed formatter installation'
fi
head -2 "$format_case/expected" > "$format_case/expected-failure"
cmp "$format_case/expected-failure" "$format_case/trace" || fail 'setup continued after formatter failure'
[[ ! -e "$slot/installed" && ! -e "$slot/install.lock" ]]
grep -Fq 'Cargo tool attempt retained:' "$format_case/setup-failure.log"
cmp "$format_case/local-before" "$format_case/.tools/rust/bin/cargo-sort"
cmp "$format_case/global-before" "$format_case/bin/cargo-sort"

python3 -m unittest discover -s "${repo_root}/scripts/ci" -p test_public_docs.py

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
    python3 - "${REPO_ROOT}/scripts/ci/public-docs-baseline.json" <<'PYDOC'
import json
import sys
# The real diagnostic parser is covered separately; this fixture checks shell
# invocation and failure propagation without invoking rustdoc.
print(json.dumps({"reason": "build-finished", "success": True}))
PYDOC
    ;;
  *) exit 2 ;;
esac
EOF
chmod +x "${public_docs_case}/bin/cargo"
# A missing diagnostic set must fail rather than passing by warning count.
if (
  cd "${repo_root}"
  TMPDIR="${public_docs_case}/tmp" PATH="${public_docs_case}/bin:${PATH}" \
    CARGO_TERM_COLOR=always REPO_ROOT="${repo_root}" \
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
if [[ "$1" == tree && -n "${FEATURE_TREE_LINE:-}" ]]; then
  if [[ "${FEATURE_TREE_SCOPE:-}" == pure && "$*" != *"--features"* \
    || "${FEATURE_TREE_SCOPE:-}" == host && "$*" == *"--features host"* ]]; then
    printf '%s\n' "$FEATURE_TREE_LINE"
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

for dependency in ic-host-artifacts ic-host-fs ic-host-process ic-host-tools \
  'ic-host-artifacts feature "archive"' 'ic-host-artifacts feature "gzip"' \
  'ic-host-artifacts feature "wasm"'; do
  scope=host
  [[ "$dependency" != ic-host-artifacts && "$dependency" != ic-host-fs ]] || scope=pure
  if TMPDIR="${feature_boundary_case}/tmp" PATH="${feature_boundary_case}/bin:${PATH}" \
    FEATURE_TREE_SCOPE="$scope" FEATURE_TREE_LINE="$dependency" \
    TRACE_FILE="${feature_boundary_case}/trace" \
    bash "${repo_root}/scripts/ci/check-library-feature-boundaries.sh" \
      >"${feature_boundary_case}/dependency-failure.log" 2>&1; then
    fail "the $scope feature boundary accepted $dependency"
  fi
  grep -Fq "unexpectedly includes $dependency" "${feature_boundary_case}/dependency-failure.log" \
    || fail "the $scope feature boundary lost the forbidden dependency diagnostic"
done

TMPDIR="${feature_boundary_case}/tmp" PATH="${feature_boundary_case}/bin:${PATH}" \
  FEATURE_TREE_SCOPE=host FEATURE_TREE_LINE=ic-host-fs TRACE_FILE="${feature_boundary_case}/trace" \
  bash "${repo_root}/scripts/ci/check-library-feature-boundaries.sh" \
    >"${feature_boundary_case}/filesystem-accepted.log" 2>&1 \
    || fail 'the cache host feature boundary rejected its current filesystem dependency'

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
  [[ "${#retained[@]}" == 1 && -d "${retained[0]}" ]] \
    || fail "$fixture discarded its failed setup"
  grep -Fq "retained: ${retained[0]}" "$retention_case/$fixture.log" \
    || fail "$fixture did not identify its retained evidence"
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
cat > "${dependency_check_case}/bin/cargo" <<'EOF'
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
  machete)
    shift
    [[ "${CARGO_NET_OFFLINE:-}" == true ]] || exit 65
    [[ "$*" == "--with-metadata" ]] || exit 65
    printf 'machete\n' >> "${TRACE_FILE}"
    ;;
  *)
    exit 66
    ;;
esac
EOF
chmod +x "${dependency_check_case}/bin/cargo"
TMPDIR="${dependency_check_case}/tmp" PATH="${dependency_check_case}/bin:${PATH}" \
  EXPECTED_TMP_ROOT="${dependency_check_case}/tmp" \
  TRACE_FILE="${dependency_check_case}/trace" \
  bash "${repo_root}/scripts/ci/check-dependencies.sh" >/dev/null
printf '%s\n' fetch audit machete > "${dependency_check_case}/expected-trace"
cmp -s "${dependency_check_case}/expected-trace" "${dependency_check_case}/trace" \
  || fail "the dependency check did not run one isolated audit before cargo machete"
[[ -z "$(find "${dependency_check_case}/tmp" -mindepth 1 -print -quit)" ]] \
  || fail "the successful dependency check left its advisory database behind"

: > "${dependency_check_case}/trace"
if TMPDIR="${dependency_check_case}/tmp" PATH="${dependency_check_case}/bin:${PATH}" \
  EXPECTED_TMP_ROOT="${dependency_check_case}/tmp" \
  TRACE_FILE="${dependency_check_case}/trace" FAIL_AUDIT=1 \
  bash "${repo_root}/scripts/ci/check-dependencies.sh" >/dev/null 2>&1; then
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
if TMPDIR="${dependency_check_case}/tmp" PATH="${dependency_check_case}/bin:${PATH}" \
  EXPECTED_TMP_ROOT="${dependency_check_case}/tmp" \
  TRACE_FILE="${dependency_check_case}/trace" FAIL_FETCH=1 \
  bash "${repo_root}/scripts/ci/check-dependencies.sh" >/dev/null 2>&1; then
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
[[ "${#preparation_logs[@]}" == 2 ]] || fail "the failed fetch discarded its preparation log"
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
