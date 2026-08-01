# IC Reporting Adapters

## Status

- Status: active direction
- Scope: read-only IC reporting expansion after 0.11
- Public boundary: typed requests, capability traits, reports, and provenance

Product coverage, priorities, and the completion bar are tracked in the
living [Roadmap to 1.0](../roadmap/1.0.md). This document owns the adapter and
collection architecture rather than duplicating milestone status.
User-facing command and collection-mode guidance lives in
[CLI Usage](../cli-usage.md).

## Decision

`ic-query` expands by authority family rather than by transport call. Each
authority family owns one built-in live adapter:

- `ic_query::ic::LiveIcSource`
- `ic_query::icrc::LiveIcrcSource`
- `ic_query::nns::LiveNnsSource`
- `ic_query::sns::LiveSnsSource`
- `ic_query::system::cmc::LiveCmcSource`

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
SNS capabilities share `SnsSourceRequest`, including explicit network and
collection provenance. SNS Root inventory and health use one
`SnsCanisterSource` capability on `LiveSnsSource` rather than separate
adapters for the Root inventory query and read-only health ingress.
Bounded swap lifecycle, sale parameters, and derived state similarly share one
`SnsSwapSource` capability rather than exposing one trait per native method.
Bounded deployed/pending and next-blessed version evidence shares one
`SnsUpgradeSource` capability across Governance and SNS-W rather than exposing
one trait per query.
Bounded proposal-window, cached treasury, voting-power, and ledger-timestamp
evidence shares one `SnsMetricsSource` capability for Governance `get_metrics`
rather than reconstructing treasury state through ledger-history scans.
Official Dashboard capabilities share `IcSourceRequest`. Canister lookup uses
focused `IcCanisterSource` and `IcCanisterCollectionSource` capabilities on
`LiveIcSource`; bounded aggregate network time series use one `IcMetricSource`
capability on that same adapter rather than one live source per metric REST
endpoint. Finite network resources use `IcNetworkSource`; the first operation
returns boundary-node data-center aggregates and the second returns bounded
daily network activity without introducing a separate adapter for either
resource.
Dashboard source-data DTOs echo that request as their source provenance, and
canister, metric, and network reports share one flattened
`IcDashboardReportProvenance`, avoiding parallel field and validation flows
without nesting the public report JSON.
Certified CMC views share one `CmcSourceRequest` and one `CmcSource`
capability on `LiveCmcSource`. The `xdr` and `cycles` reports are projections
of the same authenticated native rate rather than separate remote-method
adapters.

This keeps fixture, mirror, proxy, and pre-collected sources easy to implement
without creating a concrete live-source type for every report. Report builders
still treat capability results as untrusted boundary data: returned provenance,
canonical identifiers, requested limits, ordering, uniqueness, relation
consistency, and authority claims are validated before projection. Live
adapters validate HTTP(S) endpoint syntax before constructing their transport
or making a live call, so malformed endpoint text returns a typed error rather
than reaching an infallible parser path.

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
- SNS discovery first reads unenriched SNS-W inventory. Direct id/Root lookup
  resolves that inventory before requesting metadata for exactly one target;
  unknown lookup requests no metadata, and only `sns list` enriches every row.
- SNS Root reporting resolves one deployed SNS, uses `list_sns_canisters` as
  membership authority, and joins `get_sns_canisters_summary` health with
  `update_canister_list = false`. The sequential reads retain typed gaps and
  explicitly carry no point-in-time guarantee.
- SNS swap reporting resolves one deployed SNS and attempts exactly
  `get_lifecycle`, `get_sale_parameters`, and `get_derived_state` against its
  discovered swap canister. Component failures remain typed gaps. The adapter
  does not call the participant-bearing `get_state`, apply swap methods to
  another SNS, create a cache, or claim that the sequential responses are one
  point-in-time snapshot. Target resolution retains the existing SNS-W
  targeted discovery behavior, so the complete direct command budget is one
  SNS-W query, one selected-SNS metadata query, and three swap queries.
- SNS upgrade reporting resolves one deployed SNS, requires Governance
  `get_running_sns_version`, and compares that exact deployed version through
  SNS-W `get_next_sns_version`. A successful absent successor remains distinct
  from a typed next-version query gap. The flow makes at most four live calls
  including targeted discovery, does not read the upgrade journal, download
  Wasms, fan out, create a cache, or claim one point-in-time snapshot.
- SNS metrics reporting resolves one deployed SNS and calls Governance
  `get_metrics` with one bounded proposal-count window. The client makes three
  requests including targeted discovery; Governance performs its bounded
  latest-ledger-block lookup inside the composite query. The report preserves
  cached treasury and voting-power timestamps, never treats them as current
  ledger state, and does not enumerate transactions, fan out, create a cache,
  or claim one point-in-time snapshot.
- SNS neuron reporting preserves fixed-size native Governance fields from the
  existing `list_neurons` response, including raw dissolve state, fees,
  aging, vesting, source-NNS id, auto-stake setting, and voting multiplier.
  Bounded live rows and complete refresh pages share canonical id, timestamp,
  uniqueness, and requested-limit validation. Variable permission, followee,
  and pending-disbursement graphs remain outside collection caches and would
  require an explicit detail capability with visibility semantics.
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
- CMC system reporting makes one `get_icp_xdr_conversion_rate` query,
  authenticates the CMC certificate and certified-data witness, and proves the
  native rate leaf. The cycles view derives cycles per ICP from that same
  certified value and the documented one-trillion-cycles-per-XDR protocol
  constant; it does not scrape uncertified CMC Prometheus metrics.
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
  rather than cloning the complete snapshot before truncation. The bounded page
  builder applies the same explicit-index, canonical-cursor, requested-limit,
  uniqueness, and newest-first checks to custom page sources.
- Official Dashboard canister reporting follows one canonical canister
  principal to the bounded `/canisters/{canister_id}` REST resource, or makes
  one filtered count/page request through the official v4 collection API. A
  page is fixed to canister-id order, capped at 100 rows, and never follows a
  cursor implicitly; a returned canister principal drives the normal typed
  detail follow-up. Reports keep the API endpoint, retrieval timestamp,
  Dashboard update timestamp, and raw nullable values distinct, and explicitly
  claim neither certification nor point-in-time consistency.
- Official Dashboard metric reporting selects one documented aggregate metric,
  sends one explicit start/end/step request, caps the requested result at 1,000
  observations per series, and preserves raw named series and decimal strings.
  It does not fan out over metrics or Subnets, write a cache, or inherit
  Registry or certified-state authority.
- Official Dashboard boundary-node reporting consumes the finite v4
  data-center resource in one request. It preserves zero-node locations and raw
  owner, region, coordinate, and count strings; rows are data-center
  aggregates, not individual node identities.
- Official Dashboard daily-statistics reporting selects raw daily network
  activity from one explicitly bounded v3 request. It defaults to seven days,
  caps the window and response at 366 days/rows, tolerates missing days, and
  does not duplicate the resource's unrelated governance, supply, topology,
  or Internet Identity fields.

These flows are report-specific orchestration. There is no generic fallback
engine, dynamic Candid discovery, or implicit off-chain enrichment.

## Reporting Expansion

The official IC Dashboard documents five read-only REST API families covering
general IC state, metrics, ICRC analytics, the ICP ledger, and SNS analytics:
<https://docs.internetcomputer.org/references/ic-dashboard-api/>.

Expansion should proceed in layers:

| Priority | Reporting addition | Adapter direction |
| --- | --- | --- |
| 1 | Explicit SNS neuron permission/followee/pending-disbursement detail plus transaction-level treasury history or current-ledger verification beyond the implemented fixed-size neurons, bounded metrics, swap, and upgrade reports | Extend focused SNS capability traits on `LiveSnsSource` only where the authority, visibility, and bounds are explicit |
| 1 | NNS reward history, delegation, and governance analytics beyond the implemented native point-value and public-neuron reports | Extend focused NNS capability traits on `LiveNnsSource` |
| 2 | Individual boundary-node detail, replica-version, broader daily analytics, and trustworthy metrics beyond the implemented aggregate metric, daily-activity, and data-center sets | Extend focused capabilities on `LiveIcSource` with API endpoint/timestamp provenance |
| 2 | ICRC holders, supply history, and transaction aggregates | Add official ICRC analytics capabilities without presenting them as direct ledger state |
| 3 | Internet Identity, Bitcoin, XRC, and other protocol-canister reports beyond the implemented CMC family | Add one authority-family adapter only when multiple coherent reports justify it |

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
