#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_dir="$(mktemp -d)"
trap 'rm -rf "${work_dir}"' EXIT

fail() {
  echo "error: $*" >&2
  exit 1
}

publish_case="${work_dir}/publish"
mkdir -p "${publish_case}/bin" "${publish_case}/state"
cat > "${publish_case}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo %s\n' "$*" >> "${TRACE_FILE}"
case "${1:-}" in
  info)
    package="${2%@*}"
    if [[ "${package}" == "ic-query" && -n "${HIDE_LIBRARY_INFO:-}" ]]; then
      exit 1
    fi
    [[ -e "${STATE_DIR}/${package}" ]]
    ;;
  publish)
    package=""
    while [[ "$#" -gt 0 ]]; do
      if [[ "$1" == "-p" ]]; then
        package="${2:-}"
        break
      fi
      shift
    done
    [[ -n "${package}" ]] || exit 2
    if [[ "${package}" == "ic-query" && -n "${LIBRARY_PUBLISH_STATUS:-}" ]]; then
      exit "${LIBRARY_PUBLISH_STATUS}"
    fi
    : > "${STATE_DIR}/${package}"
    ;;
  *)
    exit 2
    ;;
esac
EOF
chmod +x "${publish_case}/bin/cargo"
current_version="$(sed -n 's/^version = "\(.*\)"/\1/p' "${repo_root}/Cargo.toml" | head -n 1)"
(
  cd "${repo_root}"
  PATH="${publish_case}/bin:${PATH}" TRACE_FILE="${publish_case}/trace" \
    STATE_DIR="${publish_case}/state" CARGO_PUBLISH_INDEX_ATTEMPTS=2 \
    CARGO_PUBLISH_INDEX_DELAY_SECONDS=0 \
    bash scripts/release/publish-workspace.sh
) >/dev/null
mapfile -t publish_trace < "${publish_case}/trace"
expected_publish_trace=(
  "cargo info ic-query@${current_version} --registry crates-io"
  "cargo publish --locked --registry crates-io -p ic-query"
  "cargo info ic-query@${current_version} --registry crates-io"
  "cargo info ic-query-cli@${current_version} --registry crates-io"
  "cargo publish --locked --registry crates-io -p ic-query-cli"
)
[[ "${#publish_trace[@]}" -eq "${#expected_publish_trace[@]}" ]] \
  || fail "the workspace publisher ran an unexpected number of Cargo commands"
for index in "${!expected_publish_trace[@]}"; do
  [[ "${publish_trace[index]}" == "${expected_publish_trace[index]}" ]] \
    || fail "the workspace publisher ran an unexpected Cargo command"
done

: > "${publish_case}/trace"
(
  cd "${repo_root}"
  PATH="${publish_case}/bin:${PATH}" TRACE_FILE="${publish_case}/trace" \
    STATE_DIR="${publish_case}/state" CARGO_PUBLISH_INDEX_ATTEMPTS=2 \
    CARGO_PUBLISH_INDEX_DELAY_SECONDS=0 \
    bash scripts/release/publish-workspace.sh
) >/dev/null
mapfile -t republish_trace < "${publish_case}/trace"
[[ "${republish_trace[0]:-}" == "cargo info ic-query@${current_version} --registry crates-io" ]] \
  || fail "the workspace publisher did not check the existing library release"
[[ "${republish_trace[1]:-}" == "cargo info ic-query-cli@${current_version} --registry crates-io" ]] \
  || fail "the workspace publisher did not check the existing CLI release"
[[ "${#republish_trace[@]}" -eq 2 ]] \
  || fail "the workspace publisher was not retry-safe for published crates"

mkdir -p "${publish_case}/failure-state"
if (
  cd "${repo_root}"
  PATH="${publish_case}/bin:${PATH}" TRACE_FILE="${publish_case}/failure-trace" \
    STATE_DIR="${publish_case}/failure-state" LIBRARY_PUBLISH_STATUS=47 \
    CARGO_PUBLISH_INDEX_ATTEMPTS=2 CARGO_PUBLISH_INDEX_DELAY_SECONDS=0 \
    bash scripts/release/publish-workspace.sh
) >/dev/null 2>&1; then
  failed_publish_status=0
else
  failed_publish_status="$?"
fi
[[ "${failed_publish_status}" -eq 47 ]] \
  || fail "the workspace publisher did not preserve a library publish failure"
[[ ! -e "${publish_case}/failure-state/ic-query-cli" ]] \
  || fail "the workspace publisher published the CLI after a library failure"

mkdir -p "${publish_case}/hidden-index-state"
if (
  cd "${repo_root}"
  PATH="${publish_case}/bin:${PATH}" TRACE_FILE="${publish_case}/hidden-index-trace" \
    STATE_DIR="${publish_case}/hidden-index-state" HIDE_LIBRARY_INFO=1 \
    CARGO_PUBLISH_INDEX_ATTEMPTS=2 CARGO_PUBLISH_INDEX_DELAY_SECONDS=0 \
    bash scripts/release/publish-workspace.sh
) >/dev/null 2>&1; then
  hidden_index_status=0
else
  hidden_index_status="$?"
fi
[[ "${hidden_index_status}" -ne 0 ]] \
  || fail "the workspace publisher accepted a library missing from the registry index"
[[ -e "${publish_case}/hidden-index-state/ic-query" ]] \
  || fail "the workspace publisher did not publish the missing library"
[[ ! -e "${publish_case}/hidden-index-state/ic-query-cli" ]] \
  || fail "the workspace publisher published the CLI before the library was indexed"
