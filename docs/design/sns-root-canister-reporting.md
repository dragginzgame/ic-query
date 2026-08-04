# SNS Root Canister Inventory and Health

## Status

- Status: implemented for 0.17
- Scope: live read-only SNS Root inventory and operational health
- Public command: `icq sns canister list <id|root-principal>`
- Public adapter capability: `SnsCanisterSource`

## Authority

The report resolves one deployed SNS through the existing SNS-W discovery
flow, then calls that SNS's Root canister through the same explicit IC API
endpoint.

Two Root methods have distinct authority:

1. `list_sns_canisters` is a query and is the membership authority for Root,
   Governance, Ledger, Swap, Index, Archive, Dapp, and Extension roles.
2. `get_sns_canisters_summary` supplies operational canister status, running
   module hash, cycles, memory, idle cycle burn, and controllers.

The summary method is a Candid update method because SNS Root must call the
management canister to collect status. `ic-query` always sends
`update_canister_list = false`. It never asks Root to poll Ledger or update its
stored archive inventory. The call is therefore an anonymous, observational
ingress call rather than mutation behavior.

## Report Contract

`SnsCanisterReport` preserves:

- the SNS-W list id, name, Root principal, endpoint, collection timestamp, and
  collector;
- native `root`, `governance`, `ledger`, `swap`, `index`, `archive`, `dapp`,
  and `extension` roles;
- native `running`, `stopping`, and `stopped` states;
- full lowercase module-hash hexadecimal text;
- raw unsigned cycles, memory bytes, and idle-cycle-burn values as canonical
  decimal strings;
- a factual `reported_zero`, `reported_nonzero`, or `unavailable` cycle-balance
  status on every inventory row, plus report-level zero and unavailable counts;
- canonical controller principals;
- the inventory method, health method, ingress call type, and explicit false
  `update_canister_list` value.

The added cycle classification, summary counts, and health-query gap replace
the report schema-1 shape in place as a pre-1.0 hard cut. No old-shape reader,
cache schema, or migration remains because this report is live-only.

Rows are ordered by native role and then canister principal. Controller
principals are ordered canonically. Duplicate inventory canister principals
are typed failures rather than ambiguous rows.

## Gaps and Consistency

The inventory query and health ingress are sequential current-state reads.
Root exposes no version that can bind them into one atomic snapshot, so every
report sets `point_in_time_guaranteed` to false.

Missing or inconsistent relations are retained as typed
`SnsCanisterGap` rows:

- missing inventory canister ids;
- missing summaries or summary canister ids;
- singleton inventory/summary id mismatches;
- summary-only canisters;
- duplicate summaries;
- matched summaries without status; and
- roles for which the health response has no status surface.

When more than one health summary claims the same inventory canister and role,
the row carries no operational status. The report retains a
`duplicate_summary` gap and never chooses health from response order.

The current Root summary response does not include Extension status. Extension
membership remains in the inventory and each Extension receives an explicit
`health_unsupported` gap.

When `list_sns_canisters` succeeds but the health ingress fails, the report
retains the authoritative inventory instead of failing the whole operation.
Every retained row has unavailable operational fields and
`cycle_balance_status: unavailable`; a typed `health_query_gap` preserves the
failed method and diagnostic reason. No `summary_missing` gaps are invented
when there was no health response to assess. Inventory-id gaps and the native
Extension `health_unsupported` gap remain visible.

## Adapter and Cache Policy

`SnsCanisterSource` extends `SnsDiscoverySource` and is implemented by the
existing `LiveSnsSource`. Discovery first fetches the unenriched SNS-W
inventory, resolves the target, and requests Governance metadata for exactly
that SNS. Custom fixture, mirror, proxy, or pre-collected sources return the
same split discovery data plus Root inventory and gap model. Report builders
canonicalize and validate custom-source rows as well as built-in live rows.
Validation requires request-matching SNS-W provenance, canonical and unique
discovery identities, exact metadata target coverage, the exact Root
method/call contract, `update_canister_list = false`, no claimed point-in-time
guarantee, canonical principals and numeric fields, and internally consistent
optional health fields. Cycle-balance status must match the raw `cycles` value,
and a health-query gap cannot coexist with returned canister status or
health-response relation gaps.

`SnsSourceRequest` carries network, endpoint, collection timestamp, and
collector. Direct built-in source calls reject a non-`ic` network before
constructing an agent.

This is a small current-state report and does not read or write a report cache.
The CLI labels it as a live operation and discloses the read-only ingress call
in command help.

## Non-Goals

- asking Root to update its archive inventory;
- direct management-canister calls;
- canister mutation or authenticated administration;
- comparing running module hashes with blessed SNS-W upgrade paths;
- swap lifecycle, treasury, or aggregate analytics;
- claiming one atomic point-in-time SNS snapshot; or
- arbitrary Candid fallback calls.
