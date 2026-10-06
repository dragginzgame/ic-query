#!/usr/bin/env bash
set -euo pipefail

version="$(perl "$(dirname "${BASH_SOURCE[0]}")/metadata.pl" version)"
tag="refs/tags/v${version}"
if ! tag_type="$(git cat-file -t "${tag}" 2>/dev/null)"; then
  echo "error: release tag v${version} does not exist" >&2
  exit 1
fi
if [[ "${tag_type}" != tag ]]; then
  echo "error: release tag v${version} must be annotated" >&2
  exit 1
fi
tag_commit="$(git rev-parse "${tag}^{commit}")"
head_commit="$(git rev-parse HEAD)"
if [[ "${tag_commit}" != "${head_commit}" ]]; then
  echo "error: release tag v${version} does not point to HEAD" >&2
  exit 1
fi
