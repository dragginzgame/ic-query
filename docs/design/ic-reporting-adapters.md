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

`ic-query` expands by authority family rather than by transport call. Built-in
host adapters group capabilities by authority:

- `ic_query::cloud_engine::LiveCloudEngineSource`
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
per-report request DTOs. Native Registry and inventory capabilities use
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
renderer. Portable proposal and public-neuron continuations share
`NnsGovernanceCollectionStatus` under Governance ownership, with one set of
persisted labels and display rules. Each collector owns its state validation,
cursor progression, and API exhaustion evidence.
Direct Governance economics, metrics, latest reward-event, and
maturity-modulation reports share one `NnsGovernanceSource` capability and the
transport-aware `NnsGovernanceRequest`; its portable async sources select
replica-query or replicated inter-canister collection. `LiveNnsSource` and
`CanisterNnsSource` implement the native and Wasm canister transports. These
remain live point-value reports rather than creating another collection cache.
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
Account/holder cursor pages and exact account detail use `IcIcrcIndexSource`
on that same Dashboard adapter; they remain bounded off-chain index reports.
Node-provider reward detail, one-page discovery, and aggregate history use
`IcNodeProviderRewardSource` on `LiveIcSource`. The CLI keeps them below the
NNS node-provider subject for entity navigation, while provenance retains the
Dashboard as authority and the source never reuses NNS inventory caches.
CloudEngine provider inventory uses `CloudEngineProviderSource` on that same
`LiveIcSource`: the CLI keeps the product surface below `cloud-engine`, while
the request and report preserve official Dashboard authority instead of
inheriting native control-plane evidence.
Explicit Type4 node inventory and exact detail use `CloudEngineNodeSource` on
the same adapter and product surface. The source echoes the reward, status,
and optional provider filters so report construction can reject a changed
query scope; it does not reuse the default-scope node-status cache.
Dashboard source-data DTOs echo that request as their source provenance, and
canister, metric, network, node-status, node-provider reward, and CloudEngine
provider/node reports share one flattened `IcDashboardReportProvenance`,
avoiding parallel field and validation flows without nesting the public report
JSON.
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

ICRC-3 history validates resident blocks and archive ranges together: coverage
must be disjoint, stay inside the requested page, and end no later than the
returned `log_length`. The [ICRC-3 specification](https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-3/README.md#icrc3_get_blocks)
defines that field as the total number of blocks. The live adapter validates
before following callbacks; custom-source report construction applies the same
range validator and checks the length when supplied. Empty pages beyond the
log end and custom sources with unknown length remain valid.

Raw ICRC numbers use plain base-10 strings, preserving arbitrary precision
and signed integer values. The shared ledger `nat_text` helper owns conversion
for account-history cursors, ledger reports, SNS token metadata, and SNS Root
canister health fields. Candid's human-facing `Display` adds underscore grouping
and is unsuitable for these
raw fields or the history validator's decimal inputs.

### Candid subtype performance follow-up: 2026-10-03

[Candid #603](https://github.com/dfinity/candid/issues/603) remains open for
deep structural equality during subtype checking. Locked Candid 0.10.37 still
performs that comparison. Its function-reference decoder invokes subtype
checking, so `Icrc3ArchiveCallback` reaches this path even when archive
following is disabled. The callback's expected recursive result schema is
fixed by the ICRC-3 wire contract; it is not caller-defined.

An offline development-build probe copied the current ICRC-3 wire definitions
and round-tripped responses with no local blocks, one range per callback, and
the same callback signature. Median decode times across 21 repetitions were:

| Callbacks | Encoded bytes | Decode time |
| --- | --- | --- |
| 1 | 154 | 0.134 ms |
| 10 | 370 | 0.159 ms |
| 100 | 2,530 | 0.627 ms |
| 1,000 | 25,004 | 5.613 ms |
| 10,000 | 250,004 | 58.267 ms |

This measures callback-count scaling for one supported schema, with debug
dependency builds, not the upstream deep-type worst case or production
latency. The larger samples scale approximately with callback count and do
not establish a local quadratic regression. Keep the upstream issue under
review when updating Candid; do not bypass callback type checking or vendor a
decoder workaround on this evidence. Unit coverage preserves multiple callback
principals, methods, and ranges through the current typed decoder. No live
ledger call or optimized-dependency benchmark was performed.

### Candid #603 candidate qualification: 2026-10-03

A subsequent isolated comparison tested Candid 0.10.37 against a candidate
that replaces the initial structural equality checks in `subtype_` and
`subtype_collect_` with pointer identity plus equality for leaf types.
`equal_impl`, global type equality, and the production dependency remain
unchanged. Separately allocated primitives must still compare equal; pointer
identity alone is insufficient with the existing subtype match arms.

Both variants passed six focused contract tests, the 16 active unit tests
included in the published Candid source, and 19 active rustdoc examples.
One upstream unit test and eight examples remain ignored. A differential
corpus of 140 types / 19,600 ordered pairs matched success/failure, ordinary
error text, collected diagnostic paths/messages, and structural-equivalence
results. Coverage includes primitive identity, shared and separately allocated
composites, nested widening and rejection, recursive type variables, function
variance, and service/ICRC-3 decoding. This is bounded fixture evidence, not
exhaustive verification of arbitrary type graphs.

Native library Clippy passed with warnings denied for both variants. Both
no-default-feature Wasm library checks compiled, with the same existing
unused-method warning in the decoder. No production dependency override was
introduced by this experiment.

Release builds used Rust 1.99.0 on x86_64 with identical dependency locks,
separate build directories, and distinct binary hashes. An initial shared-build
cache collision was detected and its measurements discarded. Timings pin one
process to CPU 2, alternate baseline/candidate ordering across six rounds,
and report the median of six process medians. Each process measures 15 samples
with iterations calibrated to about 5 ms per sample, capped at 20,000;
construction and encoding are outside the measured operation.

| Workload | Baseline | Candidate | Outcome |
| --- | --- | --- | --- |
| Nested `vec` widening, depth 16 (`nat` to `int`) | 0.336 µs | 0.145 µs | 2.3× faster |
| Nested `vec` widening, depth 64 | 3.433 µs | 0.568 µs | 6.0× faster |
| Nested `vec` widening, depth 128 | 12.984 µs | 1.278 µs | 10.2× faster |
| Nested `vec` widening, depth 256 | 62.752 µs | 2.692 µs | 23.3× faster |
| Nested `vec` widening, depth 384 | 173.986 µs | 4.033 µs | 43.1× faster |
| Collected subtype diagnostics, depth 384 widening | 174.466 µs | 4.386 µs | 39.8× faster |
| Nested `vec` rejection, depth 384 (`nat` vs `text`) | 176.481 µs | 4.802 µs | 36.8× faster |
| Separately allocated equal nested `vec` types, depth 384 | 1.061 µs | 4.072 µs | 3.8× slower |
| Sasa's service workload: 1,000 references / 20 methods | 2.712 ms | 2.778 ms | 2.4% slower |
| ICRC-3 decode, 1,000 archive callbacks | 1.012 ms | 0.986 ms | 2.6% faster |
| ICRC-3 decode, 10,000 archive callbacks | 9.671 ms | 9.448 ms | 2.3% faster |

The service workload adapts the payload and decoder from Sasa's merged
[benchmark PR #723](https://github.com/dfinity/candid/pull/723) to native timing;
the original Wasm `canbench` was not run. Small end-to-end differences do not
establish a substantial decoding improvement. The depth sweep demonstrates
the targeted repeated structural traversal and its removal for these cases,
but equal-type fast paths regress (about 6.9× at depth 128). Qualify an approach
that preserves efficient equality before adopting this candidate upstream;
these results do not justify a production dependency override in `ic-query`.

Retained local artifacts are under `target/upstream-feedback/candid-603/`:
`candidate.patch`, `probe.rs`, the two isolated sources and locks,
`method.json`, `results.json`, binary hashes, test logs, and six raw JSONL
timing files per variant. `run.py` repeats the fixture tests, differential check,
and timings offline while these artifacts remain available. These are build
artifacts and are removed by a clean.
No live ledger call or full upstream repository integration suite was run.

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
  unknown lookup requests no metadata. The resolved `MainnetSns` owns the
  original inventory id used by live reports and complete-cache refreshes.
  Lookup owns root-principal text normalization for live resolution, complete
  cache reads, and cache-status inspection.
  Direct reports and refreshes retain that row and the shared source request;
  inventory validation first requires its provenance to match the request and
  its SNS-W identity to match the mainnet constant. The temporary joined lookup
  inventory is consumed instead of retained alongside a second copy of its row.
  Only `sns list` enriches every row with Governance metadata and Swap
  `get_lifecycle`, then stores the full joined
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
- SNS proposal list and detail assembly read view options from the original
  request. Resolved live or cache context owns acquisition identity and
  metadata, keeping cache provenance independent of the current request's
  endpoint and clock.
- SNS reward checkpoint reporting strictly exhausts ordered native neuron
  pages beneath the Governance parameter ceiling and brackets the walk with
  complete parameters, latest reward event, and running version responses.
  The collector derives exhaustion from an admitted page count and absence of
  a next cursor, so initial and exhausted empty collections remain distinct
  without storing a second completion flag. Starting bracket evidence is
  validated before the walk; ending evidence must equal it in full. Final
  checkpoint validation retains the checks against collection completion
  time, and restored reports are independently validated.
  The portable neuron model owns permission and pending-disbursement checks
  shared by source admission and restored checkpoints. Exact detail still
  requires permission principals; checkpoints preserve missing principals and
  unknown codes as unassessable evidence. Each trust boundary runs those rules
  independently.
  Local diff projection treats checkpoints as untrusted, recomputes their raw
  policy and maturity evidence, and reports an allocation only after exact
  immediate-event reconciliation.
- NNS and SNS complete collections page until exhausted. Portable NNS proposal
  and public-neuron continuations expose one bounded call per advance for
  native or replicated canister collection, with caller-owned persistence and
  explicit incomplete page-limit stops.
  Native NNS refresh adapters use the portable collector's completion status
  directly, so page length and cursor evidence have one decision owner. The
  shared runner still owns page-cap enforcement, attempt writes, progress,
  failure handling, and complete-only publication orchestration.
- NNS neuron reporting follows the native ascending `get_neuron_index`
  cursor, preserves publicly readable `NeuronInfo` fields, and atomically
  publishes only an API-exhausted ordered collection. Governance exposes no
  stable collection version, so this evidence explicitly carries no
  point-in-time guarantee. List/detail reads prefer that snapshot and use
  bounded native Governance calls when it cannot satisfy the request.
- NNS Governance economics, cached metrics, latest reward event, and maturity
  modulation each preserve one canister response plus tagged replica-query or
  replicated inter-canister provenance. They do not inherit Registry versions
  or claim reward-history completeness.
- Dashboard node-provider reward reporting makes one exact, one-page, or one
  bounded aggregate-history request. It preserves raw e8s and Unix seconds,
  does not join native Governance or Registry state, and explicitly denies
  complete collection because offset pages can overlap.
- ICRC block collection validates disjoint ledger block and archive ranges
  inside the requested page before following callbacks. Following uses the
  supplied query method for one hop, at most 100 callbacks and 64 MiB of
  cumulative Candid replies admitted for decoding, with the existing 8 MiB
  per-response cap. An over-budget reply is discarded before decoding.
  Replies require unique ids inside their callback ranges; failures remain
  explicit archive error rows. Custom-source reports enforce the same page
  bounds and callback provenance before projection.
  Disjoint page ranges already establish resident uniqueness and exclude
  resident ids from followed archive rows. The custom-source duplicate set
  therefore tracks only followed archive ids, using numeric identity so
  alternate decimal spellings cannot evade it.
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
  explicitly requested index canister when supplied, fit the stated page
  capacity, and stay within any requested page cap. Failed collection
  attempts retain a resolved index canister and page/row/cursor progress when
  that evidence exists. Page validation admits canonical newest-first ids and
  exclusive boundaries before updating collection state, establishing global
  uniqueness and ordering without a final sort or duplicate scan. Local list
  projection consumes the loaded row vector rather than cloning the complete
  snapshot before truncation. Canonical row validation compares arbitrary-size
  decimal text, while caller cursor normalization retains Candid `Nat` parsing.
  The bounded page builder applies the same explicit-index, canonical-cursor,
  requested-limit, uniqueness, and newest-first checks to custom page sources.
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
  by the collected network resource; targets and `--all` are views. Aggregate
  projections count every group for snapshot-wide summaries but construct
  detailed rows and clone non-up node evidence only for selected groups. Reports
  explicitly state that the observation is uncertified, not point-in-time,
  and excludes cloud-engine nodes under the Dashboard's default scope. No
  per-row follow-up call or Registry-version claim is introduced.
  Cache admission and caller-owned snapshot projections share observation
  validation: schema, network, Dashboard authority, scope, endpoint, canonical
  timestamp, and collector identity must satisfy the same contract. Public
  projections reject certification and point-in-time claims rather than
  copying them into derived reports. Canonical-input validation proves node
  uniqueness through strict ordering; unordered source admission checks
  duplicates before sorting. Row-field validation precedes source duplicate
  detection; canonical duplicate rows fail the strict ordering check.
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
- Official Dashboard node-provider reward reporting sends one exact record,
  one page of at most 100 records, or one bounded aggregate history request.
  It preserves e8s, Unix seconds, optional proposal/Registry/XDR evidence, raw
  reward mode, and mode-specific JSON details. Offset pages can overlap even
  with the maximum reward index pinned, so it preserves source order, exposes
  an overlap warning, never auto-pages or caches, and makes no completeness or
  native Governance claim.
- Official Dashboard CloudEngine provider reporting makes one request for the
  complete node-provider resource or one exact provider. Complete collection
  is capped at 1,000 rows and validated before filtering to explicit
  CloudEngine counts or locations. Principal uniqueness is checked against
  the canonical provider ordering; location uniqueness is checked without
  reordering either location scope. Exact detail preserves valid zero-evidence
  providers. It does not infer native control-plane state, provider-to-node
  identity, health, or a join to the identifier-free boundary-location
  aggregate.
- Official Dashboard CloudEngine node reporting makes one `/nodes` request
  with the explicit `Type4` reward filter and all current status filters, or
  one exact `/nodes/{node_id}` request. Lists are capped at 10,000 rows and may
  add one exact provider filter. Reports preserve stable node/provider ids,
  raw health and assignment fields, and null CloudEngine Subnet assignments.
  The provider-to-node relation is evidence in each returned row, not an
  inferred join to provider aggregates. It remains off-chain, live-only, and
  separate from the cached Dashboard default scope.

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
| 1 | NNS reward history, delegation, and governance analytics beyond the implemented native point-value, proposal-activity, and public-neuron distribution reports | Extend focused NNS capability traits on `LiveNnsSource` |
| 2 | Broader daily analytics, API boundary-node operational/location enrichment, trustworthy running-version evidence, and trustworthy metrics beyond the implemented aggregate metric, daily-activity, data-center, certified configuration, and release-record sets | Extend the focused adapter that owns each authority; never promote Dashboard enrichment to certified state |
| 2 | CloudEngine domain/operator operational evidence and stronger authority beyond the implemented provider footprint and explicit Type4 node health/detail | Extend the focused CloudEngine source owned by each authority; do not reconcile separately timed Dashboard aggregates or promote them to native/certified state |
| 2 | ICRC circulating-supply policy, burns, and time- or kind-filtered transaction aggregates beyond the implemented scalar counts, account/holder cursor pages, exact account detail, and bounded total-supply/token-value history | Extend `IcIcrcAnalyticsSource` without presenting Dashboard or external-provider values as direct ledger state or introducing implicit enumeration |
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
