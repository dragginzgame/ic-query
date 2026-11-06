#!/usr/bin/env bash
# Shared companions: ci/cargo-release-groups.json
set -euo pipefail

# Bootstrap selection needs no checkout-local tools or full dependency graph.
# Canonical group/lock/catalog admission remains in the release adapter.
[[ $# == 2 ]] || { echo 'usage: selection.sh group|batch|version SELECTION' >&2; exit 2; }
mode="$1" selection="$2"
case "$mode" in
  group|version)
    case "$selection" in query|cli) ;; *) echo 'select RELEASE_GROUP=query or cli' >&2; exit 2 ;; esac ;;
  batch)
    case "$selection" in query|cli|query,cli) ;;
      *) echo 'select query, cli or query,cli in supplier-first order' >&2; exit 2 ;;
    esac ;;
  *) exit 2 ;;
esac
root="$(CDPATH='' cd -P -- "${BASH_SOURCE[0]%/*}/../.." && pwd -P)"
policy="$root/ci/cargo-release-groups.json"
[[ -f "$policy" && ! -L "$policy" ]] || exit 2
if [[ "$mode" == batch ]]; then
  jq -e --arg selected "$selection" '
    . as $policy | .schema == 1 and
    all($selected | split(",")[]; . as $group |
      [$policy.groups[] | select(.name == $group)] |
      length == 1 and (.[0].primary as $p | .[0].members | index($p) != null))
  ' "$policy" > /dev/null
  exit 0
fi
group="${selection%%,*}"
primary="$(jq -er --arg group "$group" '
  select(.schema == 1) | [.groups[] | select(.name == $group)] |
  select(length == 1) | .[0] | .primary as $p |
  select(.members | index($p)) | $p
' "$policy")"
[[ "$mode" == version ]] || exit 0
cd "$root"
metadata="$(cargo metadata --locked --offline --no-deps --format-version 1)"
jq -er --arg primary "$primary" --arg group "$group" --slurpfile policy "$policy" '
  . as $m | [.packages[] | select(.id as $id | $m.workspace_members | index($id))] as $members |
  [$members[] | select(.name == $primary)] | select(length == 1) | .[0].version as $version |
  select($version | test("^(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)$")) |
  select(all($policy[0].groups[] | select(.name == $group) | .members[];
    . as $name | [$members[] | select(.name == $name and .version == $version)] | length == 1)) |
  $version
' <<< "$metadata"
