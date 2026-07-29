# IC Reporting Adapters

## Status

- Status: active direction
- Scope: read-only IC reporting expansion after 0.11
- Public boundary: typed requests, capability traits, reports, and provenance

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
`ic_query::nns::NnsSourceRequest`.

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
- ICRC block collection can follow ledger-supplied archive callbacks.

These flows are report-specific orchestration. There is no generic fallback
engine, dynamic Candid discovery, or implicit off-chain enrichment.

## Reporting Expansion

The official IC Dashboard documents five read-only REST API families covering
general IC state, metrics, ICRC analytics, the ICP ledger, and SNS analytics:
<https://docs.internetcomputer.org/references/ic-dashboard-api/>.

Expansion should proceed in layers:

| Priority | Reporting addition | Adapter direction |
| --- | --- | --- |
| 1 | ICRC index account history and verified tip certificates | Extend `IcrcSource`; preserve ledger/index identity separately |
| 1 | Fuller SNS neuron state, SNS root inventory, and health | Extend the relevant SNS capability traits on `LiveSnsSource` |
| 1 | NNS neuron, economics, rewards, and governance metrics | Add focused NNS capability traits on `LiveNnsSource` |
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
