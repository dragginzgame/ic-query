#!/usr/bin/env bash
set -euo pipefail

readonly expected_missing_docs=1805
log="$(mktemp "${TMPDIR:-/tmp}/ic-query-public-docs.XXXXXX")"
trap 'rm -f -- "${log}"' EXIT

# Remove generated documentation so rustdoc emits the complete warning set on
# repeated local runs instead of reusing a fresh artifact without diagnostics.
# GitHub Actions forces colored Cargo output, so disable color for the captured
# diagnostics to keep the anchored warning count independent of the caller.
cargo clean --doc >"${log}" 2>&1
if ! CARGO_TERM_COLOR=never RUSTDOCFLAGS='-W missing-docs' \
  cargo doc -p ic-query --all-features --no-deps --locked >>"${log}" 2>&1; then
  cat "${log}" >&2
  exit 1
fi

missing_docs="$(awk '/^warning: missing documentation for / { count++ } END { print count + 0 }' "${log}")"
if [[ "${missing_docs}" -gt "${expected_missing_docs}" ]]; then
  echo "error: public documentation debt grew from ${expected_missing_docs} to ${missing_docs}" >&2
  sed -n 's/^warning: missing documentation for /missing documentation for /p' "${log}" \
    | sort \
    | uniq -c >&2
  exit 1
fi
if [[ "${missing_docs}" -lt "${expected_missing_docs}" ]]; then
  echo "error: public documentation debt fell to ${missing_docs}; lower expected_missing_docs" >&2
  exit 1
fi

echo "public documentation debt: ${missing_docs} missing-doc warnings (no growth)"
