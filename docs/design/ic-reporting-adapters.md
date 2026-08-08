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

- `ic_query::ic::LiveIcStateSource`
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
collection provenance. Complete catalog enrichment uses `SnsCatalogSource` to
add exact-target Swap lifecycles to `SnsDiscoverySource` inventory and
Governance metadata. SNS Root inventory and health use one
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
Exact neuron permission/followee detail uses `SnsNeuronSource`, while bracketed
API-exhausted maturity checkpoint collection uses `SnsRewardSource`. Both
capabilities remain on `LiveSnsSource`; variable-size detail and reward
evidence do not expand the ordinary fixed-size neuron collection cache.
Official Dashboard capabilities share `IcSourceRequest`. Canister lookup uses
focused `IcCanisterSource` and `IcCanisterCollectionSource` capabilities on
`LiveIcSource`; bounded aggregate network time series use one `IcMetricSource`
capability on that same adapter rather than one live source per metric REST
endpoint. Finite network resources use `IcNetworkSource`; the first operation
returns boundary-node data-center aggregates and the second returns bounded
daily network activity without introducing a separate adapter for either
resource. The finite Dashboard node resource uses `IcNodeStatusSource`; one
canonical raw snapshot feeds node, Subnet, and node-provider projections plus
one short-lived cache identity without placing off-chain liveness claims in
the Registry adapter. Official ICRC REST analytics use
`IcIcrcAnalyticsSource` on the same `LiveIcSource`; they do not inherit the
native `LiveIcrcSource` ledger/index authority merely because the CLI places
them below the ICRC subject.
Dashboard source-data DTOs echo that request as their source provenance, and
canister, metric, network, and node-status reports share one flattened
`IcDashboardReportProvenance`, avoiding parallel field and validation flows
without nesting the public report JSON.
Certified IC state remains a distinct authority on `LiveIcStateSource`.
`IcApiBoundaryNodeSource` returns one complete authenticated
`api_boundary_nodes` subtree with its certificate time; it does not share the
Dashboard request/provenance DTO or make Dashboard data certified.
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
than reaching an infallible parser path. The shared parser requires a
credential-free base URL with no query or fragment. Every successful
`LiveIcSource` HTTP body is capped at `MAX_IC_DASHBOARD_RESPONSE_BYTES` before
JSON decoding. Declared sizes fail before body-collection allocation, and
streamed bytes enforce the same ceiling for chunked or unknown-length
responses. Redirects are disabled so source provenance cannot silently name a
pre-redirect URL. Oversized, body-read, JSON, status, and request failures
remain distinct typed errors.

Every native agent returned by the shared builder sets an 8 MiB response-body
ceiling through `ic-agent` itself. Registry, NNS, SNS, ICRC, and CMC adapters
therefore share the same finite per-call transport policy without merging
their report-specific paging, cache, provenance, or validation contracts.

## Collection Rules

1. On-chain and Registry reports preserve their canister, endpoint, Registry
   version where applicable, collection timestamp, and exact assurance. An
   ordinary Registry query is version-consistent evidence, not certified
   evidence.
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

The Subnet Catalog makes that Registry boundary concrete. Its live collector
uses one Registry version for the Subnet list, routing table, and every Subnet
record, but labels the result `UncertifiedQuery`. Serde-facing
`RawSubnetCatalog` values become `ValidatedSubnetCatalog` only after fixed
mainnet and Registry identity, source endpoint, timestamp, raw Subnet type,
classification policy, resolver policy, canonical ordering, and payload digest
checks. The unkeyed digest detects inconsistent payloads but is not an
authenticity proof. Catalog loads take an explicit cache/network policy and
return an observable disposition; a validated route retains the exact matched
range, Registry version, digest, and provenance.
An explicit two-to-three-endpoint catalog selection may establish
`MultiEndpointAgreement` only when every distinct hostname returns the same
Registry version and canonical Registry payload. It records the agreement
digest and exact Registry query-call count, does not fall back on mismatch, and
does not claim cryptographic certification.
The NNS Registry version report independently calls
`get_certified_latest_version`, authenticates the certificate, and validates
the committed `current_version` leaf. That bounded proof does not upgrade
ordinary `get_value` evidence. `CatalogAssurance::Certified` is established
only after a complete retained delta sequence is locally reauthenticated,
replayed from version zero, matched exactly to its recomputed archive manifest,
and projected through the shared catalog validator. The authority result keeps
the validated catalog borrowed to that sealed archive; serialized provenance
cannot recreate the capability. Promotion also requires an explicit caller
observation time and maximum certificate age, retains the exact freshness
decision, and rejects stale historical authority before catalog projection.
Callers must also explicitly allow a historical pinned target or require it to
equal the newest Registry version certified by any archive batch.
The caller-runtime certified delta adapter returns at most one contiguous
batch. Chunk-referenced values named by that batch are completed with bounded,
SHA-256-verified `get_chunk` calls and exact call/byte accounting; later delta
batches are never followed implicitly. The shared ordinary `get_value` path
uses the same bounded chunk reconstruction without inheriting certified
assurance.

## Current Follow-Up Flows

- NNS exact topology follows Subnet membership through nodes and operators to
  providers at one Registry version.
- SNS discovery first reads unenriched SNS-W inventory. Direct id/Root lookup
  resolves that inventory before requesting metadata for exactly one target;
  unknown lookup requests no metadata. Only `sns list` enriches every row with
  Governance metadata and Swap `get_lifecycle`, then stores the full joined
  catalog. Its default view retains lifecycle code `3` (`committed`); `--all`
  includes every lifecycle and bounded lifecycle-query error while preserving
  SNS-W ids.
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
  and pending-disbursement graphs remain outside collection caches;
  `sns neuron info` follows one exact 32-byte neuron id through the explicit
  `SnsNeuronSource` detail capability instead.
- SNS reward checkpoint reporting strictly exhausts ordered native neuron
  pages beneath the Governance parameter ceiling and brackets the walk with
  complete parameters, latest reward event, and running version responses.
  Local diff projection treats checkpoints as untrusted, recomputes their raw
  policy and maturity evidence, and reports an allocation only after exact
  immediate-event reconciliation.
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
- Certified API boundary-node reporting performs one response-bounded
  `read_state` request for the complete `api_boundary_nodes` subtree. It
  authenticates one certificate, preserves its raw time, and projects
  canonical principal/domain/address rows at that common state-tree time. It
  makes no cache or per-node call and makes no operational, HTTP-gateway,
  ownership, or location claim.
- Official Dashboard daily-statistics reporting selects raw daily network
  activity from one explicitly bounded v3 request. It defaults to seven days,
  caps the window and response at 366 days/rows, tolerates missing days, and
  does not duplicate the resource's unrelated governance, supply, topology,
  or Internet Identity fields.
- Official Dashboard node-status reporting consumes the default public-mainnet
  `/nodes` resource in one unfiltered request capped at 10,000 rows and 8 MiB.
  It preserves raw status, assignment, alert, provider/operator, version,
  location, and hardware evidence, then projects node, Subnet, and provider
  views from one canonical snapshot. The 60-second atomic cache is keyed only
  by the collected network resource; targets and `--all` are views. Reports
  explicitly state that the observation is uncertified, not point-in-time,
  and excludes cloud-engine nodes under the Dashboard's default scope. No
  per-row follow-up call or Registry-version claim is introduced.
- Official Dashboard ICRC total-supply reporting identifies one canonical
  ledger principal and sends one explicit start/end/step request. It defaults
  to a 30-day daily window, caps requested and returned rows at 1,000,
  preserves raw unsigned-decimal base-unit values, and makes no pagination,
  ledger follow-up, enumeration, or cache call. A valid principal is not a
  claim that the Dashboard indexes that ledger, and the off-chain series does
  not replace direct current `icrc1_total_supply` evidence.
- Official Dashboard ICRC indexed-count reporting reuses the same canonical
  analytics ledger target and `IcIcrcAnalyticsSource`. One typed source flow
  selects the v2 account, holder, or transaction count resource, validates the
  exact returned kind, and preserves the non-negative total. Each operation
  performs no row request, filter, cursor traversal, per-row lookup, or cache
  write.
- Official Dashboard ICRC token-value reporting sends one explicit
  start/end/limit request for at most 90 days and 1,000 rows. It preserves the
  API's nullable legacy and explicit USD fields plus every external provider
  name and URL, validates timestamp bounds and ordering, and records possible
  truncation when the response reaches the limit. Dashboard aggregation does
  not certify, reconcile, endorse, or give on-chain authority to those external
  market values.
- Official Dashboard replica-version reporting sends one explicitly bounded
  page request or one exact detail request. It preserves the proposal-index
  ceiling, release-election fields, and Subnet rollout proposals under
  off-chain Dashboard provenance. It does not auto-page, cache, join Registry
  topology, or claim the elected release is the binary currently running.

These flows are report-specific orchestration. There is no generic fallback
engine, dynamic Candid discovery, or implicit off-chain enrichment.

## Reporting Expansion

The official IC Dashboard documents five read-only REST API families covering
general IC state, metrics, ICRC analytics, the ICP ledger, and SNS analytics:
<https://docs.internetcomputer.org/references/ic-dashboard-api/>.

Expansion should proceed in layers:

| Priority | Reporting addition | Adapter direction |
| --- | --- | --- |
| 1 | Transaction-level SNS treasury history or current-ledger verification beyond the implemented fixed-size neurons, exact neuron detail, reward checkpoints, bounded metrics, swap, and upgrade reports | Extend focused SNS capability traits on `LiveSnsSource` only where the authority, visibility, and bounds are explicit |
| 1 | NNS reward history, delegation, and governance analytics beyond the implemented native point-value and public-neuron reports | Extend focused NNS capability traits on `LiveNnsSource` |
| 2 | Broader daily analytics, API boundary-node operational/location enrichment, trustworthy running-version evidence, and trustworthy metrics beyond the implemented aggregate metric, daily-activity, data-center, certified configuration, and release-record sets | Extend the focused adapter that owns each authority; never promote Dashboard enrichment to certified state |
| 2 | ICRC account/holder rows and details, circulating-supply policy, burns, and time- or kind-filtered transaction aggregates beyond the implemented scalar counts and bounded total-supply/token-value history | Extend `IcIcrcAnalyticsSource` without presenting Dashboard or external-provider values as direct ledger state or introducing implicit enumeration |
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
