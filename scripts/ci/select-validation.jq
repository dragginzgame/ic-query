# Query owns impact selection; the shared observer owns baseline admission.
def group_targets:
  if . == "topology" then ["validate-topology", "validate-cli", "qualify-umbrella"]
  elif . == "sns-migration" then ["validate-sns", "validate-cli", "qualify-umbrella"]
  elif . == "query" then ["qualify-umbrella"]
  elif . == "cli" then ["validate-cli"]
  else error("unknown Query release group") end;
def affected:
  if startswith("crates/ic-query-topology/src/") then "topology" | group_targets
  elif startswith("crates/ic-query-sns/src/") then "sns-migration" | group_targets
  elif startswith("crates/ic-query-cli/src/") or startswith("crates/ic-query-cli/tests/") then "cli" | group_targets
  elif startswith("crates/ic-query/src/") then "query" | group_targets
  elif startswith("releases/") or . == "README.md" or . == "CHANGELOG.md" then []
  else ["workspace"] end;
if ((["validate-common"] + [$policy[0].groups[] | group_targets[]]) | unique | sort) != ($policy[0].stages | sort)
then error("Query mapping does not cover the full stage roster") else . end |
($baseline[0]) as $base |
(( $group | group_targets ) +
  [$base.changes[] | affected[]]) as $required |
((($base.available | not) or ($required | index("workspace") != null))) as $full |
{schema: 1, source: $source, group: $group,
 scope: (if $full then "workspace" else "group" end),
 policy: "ci/release-validation.json", inputs: $inputs[0],
 stages: [$policy[0].stages[] | . as $target |
   {target: $target,
    required: ($full or . == "validate-common" or ($required | index($target) != null)),
    reason: (if $base.available | not then "Full gate: " + $base.reason
      elif $full then "Full gate: shared or unclassified inputs changed since the previous group tag"
      elif . == "validate-common" then "Fresh repository admission, tools, formatting and documentation"
      elif $required | index($target) != null then "Selected group or changed package and its reverse consumers"
      else "Independent package unchanged since the previous group tag; no affected reverse-consumer contract" end)}]}
