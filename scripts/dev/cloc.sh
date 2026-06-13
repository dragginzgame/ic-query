#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if repo_root="$(git -C "${script_dir}" rev-parse --show-toplevel 2>/dev/null)"; then
    :
else
    repo_root="$(cd "${script_dir}/../.." && pwd)"
fi

crate_paths=()
if [[ -d "${repo_root}/crates" ]]; then
    for crate_path in "${repo_root}/crates"/*; do
        [[ -d "${crate_path}" ]] || continue
        crate_paths+=("${crate_path}")
    done
else
    crate_paths+=("${repo_root}")
fi

if ! command -v cloc >/dev/null 2>&1; then
    echo "error: cloc not found in PATH" >&2
    exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
    echo "error: jq not found in PATH (required for JSON parsing)" >&2
    exit 1
fi

tests_pattern='(^|/)(tests/|[^/]*tests\.rs$)'
test_attr_pattern='^[[:space:]]*#\[(tokio::)?test'
crate_column_width=24
printf -v crate_divider "%*s" "${crate_column_width}" ""
crate_divider="${crate_divider// /-}"

printf "%-*s %12s %12s %10s %9s %10s\n" \
    "${crate_column_width}" \
    "crate" \
    "runtime_loc" \
    "test_loc" \
    "test_%" \
    "test_fns" \
    "inline_fns"
printf "%-*s %12s %12s %10s %9s %10s\n" \
    "${crate_column_width}" \
    "${crate_divider}" \
    "------------" \
    "------------" \
    "--------" \
    "---------" \
    "----------"

# Count Rust test attributes and split out those hidden inside runtime files.
count_test_fns() {
    local total=0
    local inline=0
    local rust_file
    local file_count

    while IFS= read -r -d '' rust_file; do
        file_count=$(grep -Ec "${test_attr_pattern}" "${rust_file}" || true)
        total=$((total + file_count))

        if [[ ! "${rust_file}" =~ ${tests_pattern} ]]; then
            inline=$((inline + file_count))
        fi
    done < <(find "$@" -type f -name '*.rs' -print0)

    printf "%d %d\n" "${total}" "${inline}"
}

for crate_path in "${crate_paths[@]}"; do
    [[ -d "${crate_path}" ]] || continue
    crate_name="$(basename "${crate_path}")"

    source_paths=("${crate_path}")
    if [[ "${crate_path}" == "${repo_root}" ]]; then
        source_paths=()
        for source_path in "${repo_root}/src" "${repo_root}/tests"; do
            [[ -d "${source_path}" ]] || continue
            source_paths+=("${source_path}")
        done
    fi
    [[ "${#source_paths[@]}" -gt 0 ]] || continue

    # Test LOC (Rust only, path-based and intentionally delegated to cloc)
    test_loc=$(cloc "${source_paths[@]}" \
        --fullpath \
        --match-f="${tests_pattern}" \
        --include-lang=Rust \
        --json 2>/dev/null \
        | jq '.Rust.code // 0')

    # Runtime LOC (Rust only, path-based and intentionally delegated to cloc)
    runtime_loc=$(cloc "${source_paths[@]}" \
        --fullpath \
        --not-match-f="${tests_pattern}" \
        --include-lang=Rust \
        --json 2>/dev/null \
        | jq '.Rust.code // 0')

    read -r test_fns inline_test_fns < <(count_test_fns "${source_paths[@]}")
    total=$((runtime_loc + test_loc))

    if [[ "${total}" -gt 0 ]]; then
        test_pct=$(awk "BEGIN { printf \"%.1f\", (${test_loc}/${total})*100 }")
    else
        test_pct="0.0"
    fi

    printf "%-*s %12d %12d %9s%% %9d %10d\n" \
        "${crate_column_width}" \
        "${crate_name}" \
        "${runtime_loc}" \
        "${test_loc}" \
        "${test_pct}" \
        "${test_fns}" \
        "${inline_test_fns}"
done
