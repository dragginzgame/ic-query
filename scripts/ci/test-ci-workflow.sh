#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
yq="$repo_root/.tools/host/bin/yq"
jq="$repo_root/.tools/host/bin/jq"

# shellcheck disable=SC2016 # Dollar-prefixed names belong to jq.
"$yq" -o=json '.' "$repo_root/.github/workflows/ci.yml" | "$jq" -e '
  def selected($event; $ref):
    .on as $triggers |
    if $event == "push" then
      if $ref | startswith("refs/heads/") then
        any($triggers.push.branches[]?; . == ($ref | ltrimstr("refs/heads/")))
      else
        any($triggers.push.tags[]?;
          . == ($ref | ltrimstr("refs/tags/")) or
          . == "*" or (. == "v*" and ($ref | startswith("refs/tags/v"))))
      end
    else $triggers | has($event)
    end;

  . as $workflow |
  [
    ["pull_request", "refs/pull/1/merge", true],
    ["push", "refs/heads/main", true],
    ["push", "refs/heads/topic", false],
    ["push", "refs/tags/v0.52.2", false],
    ["workflow_dispatch", "refs/heads/main", true],
    ["workflow_dispatch", "refs/heads/release", true],
    ["workflow_dispatch", "refs/tags/v0.52.2", true]
  ] |
  all(.[]; . as $case | ($workflow | selected($case[0]; $case[1])) == $case[2]) and
  ($workflow.on.push == {branches:["main"]}) and
  ($workflow.jobs | keys == ["canister", "checks", "msrv"]) and
  ($workflow.jobs | all(.[];
    (has("if") | not) and
    (.strategy.matrix.host | sort == ["macos-15", "macos-15-intel", "ubuntu-24.04"]) and
    all(.steps[]; .with.ref == null))) and
  ($workflow.jobs.canister.steps | any(.[]; .if == "always()" and ((.uses // "") | startswith("actions/upload-artifact@")))) and
  ($workflow.jobs.checks.steps | any(.[]; .if == "failure()" and ((.uses // "") | startswith("actions/upload-artifact@"))))
' > /dev/null
echo 'CI event selection and nine-job native coverage fixtures passed'
