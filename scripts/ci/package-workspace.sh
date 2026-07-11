#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${repo_root}"

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"
if [[ -z "${version}" ]]; then
  echo "error: failed to read package version from Cargo.toml" >&2
  exit 1
fi

bash scripts/ci/cargo-package-retry.sh -p ic-query --locked "$@"
bash scripts/ci/cargo-package-retry.sh -p ic-query-cli --locked --no-verify "$@"

library_crate="target/package/ic-query-${version}.crate"
cli_crate="target/package/ic-query-cli-${version}.crate"
if [[ ! -f "${library_crate}" || ! -f "${cli_crate}" ]]; then
  echo "error: expected workspace package archives were not created" >&2
  exit 1
fi

verify_root="$(mktemp -d)"
trap 'rm -rf "${verify_root}"' EXIT
tar -xzf "${library_crate}" -C "${verify_root}"
tar -xzf "${cli_crate}" -C "${verify_root}"
mkdir -p "${verify_root}/.cargo"
cat > "${verify_root}/.cargo/config.toml" <<EOF
[patch.crates-io]
ic-query = { path = "ic-query-${version}" }
EOF

(
  cd "${verify_root}/ic-query-cli-${version}"
  cargo update -p ic-query --offline
  cargo check --locked --offline --all-targets
)
