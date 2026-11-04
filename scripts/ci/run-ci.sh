#!/usr/bin/env bash
# Shared companions: scripts/ci/validation-evidence.pl scripts/ci/run-validation-targets.sh ci/release-validation.json ci/cargo-release-groups.json
set -euo pipefail
root="$(CDPATH='' cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$root"
[[ ! -L target && ( ! -e target || -d target ) ]] || { echo 'CI target directory is redirected' >&2; exit 1; }
mkdir -p target
directory="$(mktemp -d target/ci-roster.XXXXXX)"
jq -cS '[.groups[].name]' ci/cargo-release-groups.json > "$directory/groups.json"
# The group projection and policy are admitted by Shared, not a second Make list.
perl scripts/ci/validation-evidence.pl roster "$root" ci/release-validation.json "$directory/groups.json" > "$directory/stages"
targets=()
while IFS= read -r target; do targets+=("$target"); done < "$directory/stages"
rm -f -- "$directory/groups.json" "$directory/stages"
rmdir -- "$directory"
bash scripts/ci/run-validation-targets.sh --fail-fast --input ci/release-validation.json --input scripts/ci/run-ci.sh "$@" "${targets[@]}"
