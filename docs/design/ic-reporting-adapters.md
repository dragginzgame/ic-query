# IC Reporting Adapters

## Status

- Status: active direction
- Scope: read-only IC reporting expansion after 0.11
- Public boundary: typed requests, capability traits, reports, and provenance

Product coverage, priorities, and the completion bar are tracked in the
living [Roadmap to 1.0](../roadmap/1.0.md). This document owns the adapter and
collection architecture rather than duplicating milestone status.

## Decision

`ic-query` expands by authority family rather than by transport call. Each
authority family owns one built-in live adapter:

- `ic_query::nns::LiveNnsSource`
- `ic_query::sns::LiveSnsSource`
- `ic_query::icrc::LiveIcrcSource`

Report families continue to own small source capability traits. A custom
adapter implements only the capabilities it can supply, while the built-in
adapter implements all capabilities supported for its authority. Identical
network and collection provenance use a shared source request instead of
per-report request DTOs. NNS capabilities use
`ic_query::nns::NnsSourceRequest`. Registry-derived NNS inventory operations
share `NnsInventoryCacheRequest`, `NnsInventoryListRequest`,
`NnsInventoryInfoRequest`, and `NnsInventoryRefreshRequest`. Simple
ledger-wide ICRC capabilities share `IcrcLedgerRequest`. SNS neuron and
proposal cache inspection shares the `SnsCache*` request and report contracts;
the collection-specific builders still own their distinct cache paths and
rendering. Complete NNS Governance proposal and neuron collections share
`NnsGovernanceRefreshRequest`, `NnsGovernanceCacheRequest`,
`NnsGovernanceRefreshAttemptStatus`, and `NnsGovernanceQueryError`, while each
capability retains its own page validation, cache identity, report, and
renderer. Direct Governance economics, metrics, latest reward-event, and
maturity-modulation reports share one `NnsGovernanceSource` capability and the
same `NnsSourceRequest`; they remain live point-value reports rather than
creating another complete-collection cache.

This keeps fixture, mirror, proxy, and pre-collected sources easy to implement
without creating a concrete live-source type for every report.

## Collection Rules

1. On-chain and Registry reports preserve their authoritative canister,
   endpoint, Registry version where applicable, and collection timestamp.
2. Indexed or REST-derived analytics identify their index/API endpoint and
   timestamp. They do not inherit an on-chain Registry version.
3. External enrichment remains a separate field or report with its own
   provenance. A join never makes external data authoritative IC state.
4. Follow-up collection is explicit and typed. Discovery reports expose stable
   identifiers, and a follow-up builder accepts those identifiers in a normal
   request. Arbitrary Candid calls are not report adapters.
5. Partial follow-up failures are typed gaps or per-target failures. They are
   not silently dropped.
6. Cache identity describes collected data. View filters, sorting, and limits
   do not create alternate complete-snapshot identities.

## Current Follow-Up Flows

- NNS exact topology follows Subnet membership through nodes and operators to
  providers at one Registry version.
- SNS discovery follows SNS-W results with per-SNS metadata calls.
- NNS and SNS complete collections page until exhausted.
- NNS neuron reporting follows the native ascending `get_neuron_index`
  cursor, preserves publicly readable `NeuronInfo` fields, and atomically
  publishes only an API-exhausted ordered collection. Governance exposes no
  stable collection version, so this evidence explicitly carries no
  point-in-time guarantee. List/detail reads prefer that snapshot and use
  bounded native Governance calls when it cannot satisfy the request.
- NNS Governance economics, cached metrics, latest reward event, and maturity
  modulation each preserve one native canister response plus endpoint and
  collection provenance. They do not inherit Registry versions or claim
  reward-history completeness.
- ICRC block collection can follow ledger-supplied archive callbacks.
- ICRC tip-certificate collection authenticates the certificate and proves the
  ledger tip witness against the canister's certified-data value.
- ICRC account history resolves an index through ICRC-106 or an explicit
  canister id, verifies the index's ledger identity, and paginates backward
  with an exclusive transaction-id cursor. The same capability decodes the
  official generic index-ng and deployed ICP index interfaces without
  conflating structured ICRC accounts with ICP account identifiers. Complete
  collection resolves and verifies that context once, exhausts the same index,
  and atomically publishes one endpoint/ledger/account snapshot. It records
  API exhaustion but no point-in-time guarantee because the index exposes no
  snapshot version. A custom complete-collection source must return the
  explicitly requested index canister when supplied. Failed collection
  attempts retain a resolved index canister and page/row/cursor progress when
  that evidence exists. The collector canonicalizes transaction ids and checks
  adjacent duplicates after the final ordering pass, avoiding a second
  full-history id set; local list projection consumes the loaded row vector
  rather than cloning the complete snapshot before truncation.

These flows are report-specific orchestration. There is no generic fallback
engine, dynamic Candid discovery, or implicit off-chain enrichment.

## Reporting Expansion

The official IC Dashboard documents five read-only REST API families covering
general IC state, metrics, ICRC analytics, the ICP ledger, and SNS analytics:
<https://docs.internetcomputer.org/references/ic-dashboard-api/>.

Expansion should proceed in layers:

| Priority | Reporting addition | Adapter direction |
| --- | --- | --- |
| 1 | Fuller SNS neuron state, SNS root inventory, and health | Extend the relevant SNS capability traits on `LiveSnsSource` |
| 1 | NNS reward history, delegation, and governance analytics beyond the implemented native point-value and public-neuron reports | Extend focused NNS capability traits on `LiveNnsSource` |
| 2 | Canister, boundary-node, replica-version, and network metrics | Add an official Dashboard family adapter with API endpoint/timestamp provenance |
| 2 | ICRC holders, supply history, and transaction aggregates | Add official ICRC analytics capabilities without presenting them as direct ledger state |
| 3 | CMC/XDR, Internet Identity, Bitcoin, and other protocol-canister reports | Add one authority-family adapter only when multiple coherent reports justify it |

New report work first identifies whether its authority is a canister, Registry
snapshot, certified response, official index, official REST API, or external
enrichment. That decision determines its adapter, provenance, cache, and
validation contract.

## Non-Goals

- one universal trait containing every IC query;
- one concrete adapter type per report;
- arbitrary unaudited Candid invocation;
- authenticated mutation or management-canister operations;
- merging Dashboard analytics into exact Registry evidence;
- compatibility aliases for replaced pre-1.0 adapter names.
