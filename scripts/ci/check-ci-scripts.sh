#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
make_bin="$(command -v make)"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/ic-query-ci-scripts.XXXXXX")"
trap 'rm -rf -- "${work_dir}"' EXIT

fail() {
  echo "error: $*" >&2
  exit 1
}

ci_gate_case="${work_dir}/ci-gate"
mkdir -p "${ci_gate_case}/bin"
cat > "${ci_gate_case}/bin/make" <<'EOF'
#!/usr/bin/env bash
printf 'make %s\n' "$*" >> "${TRACE_FILE}"
EOF
chmod +x "${ci_gate_case}/bin/make"
(
  cd "${repo_root}"
  TRACE_FILE="${ci_gate_case}/trace" \
    "${make_bin}" --no-print-directory MAKE="${ci_gate_case}/bin/make" ci
) >/dev/null
mapfile -t ci_gate_trace < "${ci_gate_case}/trace"
expected_ci_targets=(
  changelog-check
  actions-check
  package-contents-check
  feature-boundary-check
  library-process-boundary-check
  ci-scripts-check
  publish-guards-check
  release-guards-check
  type-docs-check
  public-docs-check
  dependency-check
  schema-version-check
  fmt-check
  check
  clippy
  test
  package
)
[[ "${#ci_gate_trace[@]}" -eq "${#expected_ci_targets[@]}" ]] \
  || fail "make ci ran an unexpected number of targets"
for index in "${!expected_ci_targets[@]}"; do
  expected_command="make --no-print-directory ${expected_ci_targets[index]}"
  [[ "${ci_gate_trace[index]}" == "${expected_command}" ]] \
    || fail "make ci did not run its targets sequentially"
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
cat > "${schema_version_case}/future.rs" <<'EOF'
pub const CACHE_SCHEMA_VERSION: u32 = 2;
EOF
if bash "${repo_root}/scripts/ci/check-schema-versions.sh" "${schema_version_case}" \
  >/dev/null 2>&1; then
  fail "the schema-version check accepted a pre-1.0 version bump"
fi

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

public_docs_case="${work_dir}/public-docs"
mkdir -p "${public_docs_case}/bin"
public_docs_warning_count="$(sed -n \
  's/^readonly expected_missing_docs=\([0-9][0-9]*\)$/\1/p' \
  "${repo_root}/scripts/ci/check-public-docs.sh")"
[[ "${public_docs_warning_count}" =~ ^[0-9]+$ ]] \
  || fail "the public documentation check has no numeric warning baseline"
cat > "${public_docs_case}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  clean)
    ;;
  doc)
    for ((warning = 0; warning < WARNING_COUNT; warning++)); do
      if [[ "${CARGO_TERM_COLOR:-}" == "always" ]]; then
        printf '\033[33mwarning: missing documentation for a struct\033[0m\n' >&2
      else
        printf 'warning: missing documentation for a struct\n' >&2
      fi
    done
    ;;
  *)
    exit 2
    ;;
esac
EOF
chmod +x "${public_docs_case}/bin/cargo"
(
  cd "${public_docs_case}"
  PATH="${public_docs_case}/bin:${PATH}" CARGO_TERM_COLOR=always \
    WARNING_COUNT="${public_docs_warning_count}" \
    bash "${repo_root}/scripts/ci/check-public-docs.sh"
) >/dev/null 2>&1 \
  || fail "the public documentation check is not stable under forced Cargo color"

feature_boundary_case="${work_dir}/feature-boundary"
mkdir -p "${feature_boundary_case}/bin" "${feature_boundary_case}/tmp"
cat > "${feature_boundary_case}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
if [[ -n "${FAIL_FEATURE_CHECK:-}" && "$*" == *"--features host"* ]]; then
  exit 51
fi
EOF
chmod +x "${feature_boundary_case}/bin/cargo"
TMPDIR="${feature_boundary_case}/tmp" PATH="${feature_boundary_case}/bin:${PATH}" \
  bash "${repo_root}/scripts/ci/check-library-feature-boundaries.sh" >/dev/null \
  || fail "the feature-boundary check rejected successful Cargo commands"
[[ -z "$(find "${feature_boundary_case}/tmp" -mindepth 1 -print -quit)" ]] \
  || fail "the successful feature-boundary check left temporary files"
if TMPDIR="${feature_boundary_case}/tmp" PATH="${feature_boundary_case}/bin:${PATH}" \
  FAIL_FEATURE_CHECK=1 \
  bash "${repo_root}/scripts/ci/check-library-feature-boundaries.sh" >/dev/null 2>&1; then
  fail "the feature-boundary check hid a failed Cargo command"
fi
[[ -z "$(find "${feature_boundary_case}/tmp" -mindepth 1 -print -quit)" ]] \
  || fail "the failed feature-boundary check left temporary files"

package_retry_case="${work_dir}/package-retry"
mkdir -p "${package_retry_case}/bin"
cat > "${package_retry_case}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
exit 42
EOF
chmod +x "${package_retry_case}/bin/cargo"
if PATH="${package_retry_case}/bin:${PATH}" CARGO_PACKAGE_RETRIES=1 \
  bash "${repo_root}/scripts/ci/cargo-package-retry.sh" --workspace --locked \
  >/dev/null 2>&1; then
  package_status=0
else
  package_status="$?"
fi
[[ "${package_status}" -eq 42 ]] \
  || fail "the package retry wrapper did not preserve Cargo's failure status"

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
EOF
chmod +x "${package_workspace_case}/bin/cargo"
PATH="${package_workspace_case}/bin:${PATH}" \
  TRACE_FILE="${package_workspace_case}/trace" CARGO_PACKAGE_RETRIES=1 \
  bash "${repo_root}/scripts/ci/package-workspace.sh" >/dev/null
mapfile -t package_workspace_trace < "${package_workspace_case}/trace"
[[ "${package_workspace_trace[0]:-}" == "cargo package -p ic-query --locked" ]] \
  || fail "the workspace package check did not package the library first"
expected_cli_package='cargo package -p ic-query-cli --locked --config patch.crates-io.ic-query.path="crates/ic-query"'
[[ "${package_workspace_trace[1]:-}" == "${expected_cli_package}" ]] \
  || fail "the workspace package check did not verify the CLI against the unpublished local library"
[[ "${#package_workspace_trace[@]}" -eq 2 ]] \
  || fail "the workspace package check ran unexpected Cargo commands"
