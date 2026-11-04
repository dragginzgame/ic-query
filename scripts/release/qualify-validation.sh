#!/usr/bin/env bash
# Shared companions: scripts/release/validation.sh
set -euo pipefail
root="$(CDPATH='' cd -P -- "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$root"
for directory in target target/release-validation; do
  [[ ! -L "$directory" && ( ! -e "$directory" || -d "$directory" ) ]] \
    || { echo 'qualification output directory is redirected' >&2; exit 1; }
done
export RELEASE_SOURCE RELEASE_GROUP RELEASE_PREVIOUS
RELEASE_SOURCE="$(git rev-parse HEAD)"
mkdir -p target/release-validation
directory="$(mktemp -d target/release-validation/qualification.XXXXXX)"
export RELEASE_GROUP=query
RELEASE_PREVIOUS="$(bash scripts/ci/cargo-release-group.sh version query)"
bash scripts/release/validation.sh qualify "$directory/workspace" workspace
for group in query cli; do
  export RELEASE_GROUP="$group"
  RELEASE_PREVIOUS="$(bash scripts/ci/cargo-release-group.sh version "$group")"
  bash scripts/release/validation.sh qualify "$directory/$group" qualification
done
full="$(cat "$directory/workspace")"
for group in query cli; do
  selected="$(cat "$directory/$group")"
  jq -e --arg group "$group" --slurpfile scopes ci/release-validation-scopes.json \
    --slurpfile full "$full/proof.json" --slurpfile context "$selected/context.json" '
    .source == $full[0].source and .scope == "group" and
    ([.stages[]|select(.state == "not-applicable")|.target] | sort) ==
      ($scopes[0].groups[$group].omit | keys) and
    ($context[0].purpose == "qualification")
  ' "$selected/proof.json" > /dev/null
  cmp "$full/context.json" <(jq -cS '.group="query"' "$selected/context.json")
done
jq -ncS --slurpfile context "$full/context.json" --arg source "$RELEASE_SOURCE" \
  --arg full "$full" --arg query "$(cat "$directory/query")" --arg cli "$(cat "$directory/cli")" \
  '{schema:1,source:$source,host:$context[0].host,mapping:$context[0].mapping,runs:{workspace:$full,query:$query,cli:$cli}}' \
  > "$directory/qualification.json"
printf 'Matched full/selected qualification retained: %s/qualification.json\n' "$directory"
