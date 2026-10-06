#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
source "$repo_root/ci/tool-versions.env"
case "$(uname -s):$(uname -m)" in
  Linux:x86_64|Linux:amd64) checksum="$IC_QUERY_YQ_SHA256_LINUX_AMD64" ;;
  Darwin:x86_64|Darwin:amd64) checksum="$IC_QUERY_YQ_SHA256_DARWIN_AMD64" ;;
  Darwin:arm64|Darwin:aarch64) checksum="$IC_QUERY_YQ_SHA256_DARWIN_ARM64" ;;
  *) echo 'unsupported IC Query tooling host' >&2; exit 1 ;;
esac
exec bash "$repo_root/scripts/ci/install-yq.sh" \
  --version "$IC_QUERY_YQ_VERSION" --sha256 "$checksum" \
  --install-dir "${1:?expected parser installation directory}"
