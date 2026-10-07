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

check_make_execution_modes() {
  local fixture="$work_dir/make-execution" mode status
  mkdir -p "$fixture"
  cp "$repo_root/Makefile" "$fixture/Makefile"
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
  status=0
  env -u MAKEFLAGS -u MFLAGS -u MAKEOVERRIDES -u GNUMAKEFLAGS \
    TRACE_FILE="$fixture/trace" "$make_bin" --no-print-directory -C "$fixture" \
    -i MAKEFLAGS=--no-print-directory execution-probe > "$fixture/blocked.log" 2>&1 || status=$?
  [[ "$status" == 2 && ! -e "$fixture/trace" ]] || fail 'MAKEFLAGS override hid invocation mode'
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
  changelog-check
  shared-tooling-check
  host-tools-check
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
  printf 'make --no-print-directory -C %s %s\n' "${repo_root}" "${target}"
done > "${ci_gate_case}/expected-trace"
cmp -s "${ci_gate_case}/expected-trace" "${ci_gate_case}/trace" \
  || fail "make ci ran an unexpected target sequence"

for failed_target in changelog-check dependency-pins-check doc-links-check test; do
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
    printf 'make --no-print-directory -C %s %s\n' "${repo_root}" "${target}"
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

pin_check_case="${work_dir}/pin-check"
mkdir -p "$pin_check_case/scripts/ci"
cp "$repo_root/Makefile" "$pin_check_case/Makefile"
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
cp "$repo_root/Makefile" "$version_case/Makefile"
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
cp "$repo_root/Makefile" "$offline_validation_case/Makefile"
cp "$repo_root/scripts/ci/package-workspace.sh" "$offline_validation_case/scripts/ci/"
cat > "$offline_validation_case/bin/git" <<'EOF'
#!/usr/bin/env bash
case "$*" in
  'ls-files --others --exclude-standard' | 'diff-index --quiet HEAD --') exit 0 ;;
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
cp "$repo_root/Makefile" "$tools_case/Makefile"
for family in host ic; do
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
  printf '%s\n' "host --versions $tools_case/ci/tool-versions.env --with-ripgrep$suffix" \
    "ic --pins $tools_case/ci/ic-tools.tsv$suffix" > "$tools_case/expected"
  cmp -s "$tools_case/expected" "$tools_case/trace" \
    || fail "local tool setup/check changed ordering, pins or offline selection"
  : > "$tools_case/trace"
  if TRACE_FILE="$tools_case/trace" EXPECTED_ROOT="$tools_case" FAIL_FAMILY=host \
    MAKEFLAGS='' MAKEOVERRIDES='' IC_TOOL_PINS="$tools_case/ci/ic-tools.tsv" \
    HOST_TOOL_VERSIONS="$tools_case/ci/tool-versions.env" \
    "$make_bin" --no-print-directory -C "$tools_case" "$target" >/dev/null 2>&1; then
    fail "tool setup/check accepted a host failure"
  fi
  printf '%s\n' "host --versions $tools_case/ci/tool-versions.env --with-ripgrep$suffix" > "$tools_case/expected"
  cmp -s "$tools_case/expected" "$tools_case/trace" \
    || fail "tool setup/check continued after a failed host command"
done

format_case="$work_dir/format"
mkdir -p "$format_case/bin" "$format_case/ci" "$format_case/scripts/ci" "$format_case/scripts/dev"
cp "$repo_root/Makefile" "$format_case/Makefile"
cp "$repo_root/scripts/ci/check-format-tools.sh" "$format_case/scripts/ci/"
printf 'export SHARED_TOOLING_CARGO_SORT_VERSION=9.8.7\n' > "$format_case/ci/tool-versions.env"
cat > "$format_case/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" != install ]]; then
  [[ "$CARGO_NET_OFFLINE" == true && "$RUSTUP_AUTO_INSTALL" == 0 ]] || exit 79
fi
printf '%s\n' "$*" >> "$TRACE_FILE"
case "$*" in
  'sort --version') echo 'cargo-sort 9.8.7' ;;
  'fmt --version') echo rustfmt ;;
esac
[[ "$*" != "${FAIL_FORMAT_COMMAND:-}" ]] || exit 78
EOF
cat > "$format_case/scripts/dev/install-host-tools.sh" <<'EOF'
#!/usr/bin/env bash
printf 'host setup %s\n' "$*" >> "$TRACE_FILE"
EOF
chmod +x "$format_case/bin/cargo"
for target in fmt fmt-check; do
  : > "$format_case/trace"
  PATH="$format_case/bin:$PATH" TRACE_FILE="$format_case/trace" \
    CARGO_NET_OFFLINE=false RUSTUP_AUTO_INSTALL=1 MAKEFLAGS='' MAKEOVERRIDES='' \
    HOST_TOOL_VERSIONS="$format_case/ci/tool-versions.env" \
    "$make_bin" --no-print-directory -C "$format_case" "$target" >/dev/null
  if [[ "$target" == fmt ]]; then sort_command='sort --workspace'; fmt_command='fmt --all';
  else sort_command='sort --workspace --check'; fmt_command='fmt --all -- --check'; fi
  printf '%s\n' 'sort --version' 'fmt --version' "$sort_command" "$fmt_command" > "$format_case/expected"
  cmp "$format_case/expected" "$format_case/trace" \
    || fail "$target changed formatter order, check flags or the selected setup pin"
  for command in 'sort --version' 'fmt --version' "$sort_command" "$fmt_command"; do
    : > "$format_case/trace"
    if PATH="$format_case/bin:$PATH" TRACE_FILE="$format_case/trace" FAIL_FORMAT_COMMAND="$command" \
      MAKEFLAGS='' MAKEOVERRIDES='' HOST_TOOL_VERSIONS="$format_case/ci/tool-versions.env" \
      "$make_bin" --no-print-directory -C "$format_case" "$target" >/dev/null 2>&1; then
      fail "$target accepted a failed $command"
    fi
    awk -v stop="$command" '{print; if ($0 == stop) exit}' "$format_case/expected" > "$format_case/expected-failure"
    cmp "$format_case/expected-failure" "$format_case/trace" \
      || fail "$target continued after failed formatter admission or execution"
  done
done
: > "$format_case/trace"
PATH="$format_case/bin:$PATH" TRACE_FILE="$format_case/trace" \
  MAKEFLAGS='' MAKEOVERRIDES='' HOST_TOOL_VERSIONS="$format_case/ci/tool-versions.env" \
  "$make_bin" --no-print-directory -C "$format_case" install-dev \
  CARGO_AUDIT_VERSION=8.7.6 CARGO_MACHETE_VERSION=7.6.5 >/dev/null
printf '%s\n' "host setup --versions $format_case/ci/tool-versions.env --with-ripgrep" \
  'install --locked cargo-sort --version 9.8.7' \
  'install --locked cargo-audit --version 8.7.6' \
  'install --locked cargo-machete --version 7.6.5' > "$format_case/expected"
cmp "$format_case/expected" "$format_case/trace" \
  || fail 'install-dev did not share the formatter pin or changed development setup ordering'

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
  [[ "$dependency" != ic-host-artifacts ]] || scope=pure
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
