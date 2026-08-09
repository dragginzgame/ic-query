# Library Usage

This guide is for Rust crates that want to call `ic-query` directly instead of
spawning the `icq` executable.

The usual downstream shape is:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["host"] }
```

Use `host` for native tools that need live calls, filesystem caches, refresh
operations, or cache-backed report builders. The library has no CLI feature;
`icq` parsing and dispatch are owned by `ic-query-cli`.

The focused host choices are `cloud-engine-host`, `cmc-host`,
`certified-subnet-catalog-host`, `dashboard-host`, `ic-state-host`, `icrc-host`,
`nns-host`, `nns-topology-host`, `sns-host`, and `subnet-catalog-host`. The
complete `host` feature is their convenience union.
Both NNS subsets are nested under `nns-host`, and
`certified-subnet-catalog-host` and `nns-topology-host` each include
`subnet-catalog-host`.

For official Dashboard REST reports, node-provider rewards, CloudEngine
provider and Type4 node collection, and the shared observed default-scope
node-status cache, use the independent Dashboard feature:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["dashboard-host"] }
```

`dashboard-host` exposes `LiveIcSource`, Dashboard custom-source traits and
builders including `IcNodeProviderRewardSource`, `CloudEngineNodeSource`, and
`CloudEngineProviderSource`, and the confined node-status cache. It enables
Reqwest, Tokio, URL,
and the capability-filesystem dependencies required by those APIs. It does not
enable `ic-agent`, Registry protobuf decoding, `serde_cbor`, or native
NNS/SNS/ICRC host adapters. Reqwest can retain cryptographic implementations
such as SHA-256 transitively; dependency checks distinguish those transitives
from ic-query's own direct optional edges.

For the certified mainnet API boundary-node state tree, use the independent IC
state feature:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["ic-state-host"] }
```

`ic-state-host` exposes `LiveIcStateSource`, `IcApiBoundaryNodeSource`, the
live/custom-source builders, and their source-data contract. It directly
enables `ic-agent`, Tokio, and URL validation. It does not enable the confined
cache filesystem, Dashboard Reqwest transport, Registry Prost decoding,
Futures collection, direct CBOR decoding, or direct SHA-256 hashing. Reqwest,
CBOR, and cryptographic packages remain transitive through `ic-agent`.

For authenticated Cycle Minting Canister ICP/XDR and cycles reports, use:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["cmc-host"] }
```

`cmc-host` exposes `LiveCmcSource`, `CmcSource`, report builders, and certified
rate evidence. It directly enables `ic-agent`, Tokio, URL, and CBOR certificate
and witness decoding. It does not enable confined-cache dependencies, Registry
Prost decoding, or direct Futures, Reqwest, or SHA-256 edges. Reqwest and
cryptographic packages remain transitive through `ic-agent`.

For public CloudEngine operator and marketplace reports, use:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["cloud-engine-host"] }
```

`cloud-engine-host` exposes `LiveCloudEngineSource`, `CloudEngineSource`, and
the operator/price report builders. Combine it with `subnet-catalog-host` for
the Registry-backed list builder and its focused operator-binding source trait;
the convenience `host` feature already includes both. `cloud-engine-host`
directly enables `ic-agent`, Tokio, and URL validation. It does not directly
enable cache-filesystem dependencies,
Registry Prost decoding, Futures fan-out, Reqwest REST transport, CBOR
certification, or SHA-256 hashing. Packages in the latter groups can remain
transitive through `ic-agent`.

For the Registry-backed CloudEngine inventory, enable both authority features
(or use `host`):

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["cloud-engine-host", "subnet-catalog-host"] }
```

For native ICRC ledger/index reports, certified-tip verification, and complete
account-history caches, use:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["icrc-host"] }
```

`icrc-host` exposes `LiveIcrcSource`, its report-specific source traits and
builders, certificate/hash-tree verification, and the confined complete
account-history cache. It enables `ic-agent`, Futures, Tokio, URL, SHA-256,
CBOR, and the capability-filesystem dependencies required by those APIs. It
does not enable Dashboard, Registry, NNS, or SNS host adapters and has no
direct ic-query dependency on Reqwest or Prost. Reqwest remains transitive
through `ic-agent`.

For native SNS discovery, targeted reports, complete proposal/neuron caches,
reward checkpoints, and local checkpoint diffs, use:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["sns-host"] }
```

`sns-host` exposes `LiveSnsSource`, its report-specific source traits and
builders, confined caches, refresh-attempt evidence, progress events, and
reward evidence. It enables `ic-agent`, Futures, Tokio, URL, and the
capability-filesystem dependencies required by those APIs. It shares only the
ICRC token-metadata query mechanics required by SNS ledger reports; native
ICRC transaction, archive, certificate, and account-history adapters remain
disabled. It has no direct ic-query dependency on Reqwest, Prost, CBOR, or
SHA-256, although `ic-agent` retains Reqwest, CBOR, and cryptographic packages
transitively.

For a native embedder that needs only the live/cache Subnet catalog API, use
the narrower feature:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["subnet-catalog-host"] }
```

`subnet-catalog-host` includes the IC agent, Registry protobuf decoding,
hashing, caller-runtime async APIs, the synchronous Tokio bridge,
capability-rooted cache IO through
`cap-std`/`cap-fs-ext`, and endpoint validation required by
`ic_query::subnet_catalog`. It does not enable `ic-query`'s direct optional
Dashboard `reqwest` transport or `serde_cbor` certification dependencies. Those
packages may still appear transitively through `ic-agent`. The full `host`
feature is a strict superset.

For certified Registry archive/replay and archive-bound Subnet Catalog
authority without the complete NNS host surface, use:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["certified-subnet-catalog-host"] }
```

`certified-subnet-catalog-host` includes `subnet-catalog-host` and adds the
bounded certified delta, archive, replay, projection, and certified catalog
cache APIs. It directly enables CBOR certificate decoding. It does not enable
NNS Governance, proposal, neuron, component-inventory, or derived-topology
host adapters, and it does not add ic-query's direct Dashboard Reqwest edge.
Network calls remain confined to explicitly selected certified collection or
archive refresh operations; archive load, replay, projection, and cache load
are local-only.

For the Subnet Catalog plus exact-version joined NNS Subnet topology, use:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["nns-topology-host"] }
```

`nns-topology-host` exposes the joined topology live source, strict cache load,
explicit refresh, refresh-missing, and refresh-stale APIs. It includes
`subnet-catalog-host` and shares its Registry agent, runtime, and confined-cache
substrate. It does not enable ic-query's direct optional Dashboard Reqwest or
CBOR certification dependencies, and it does not expose the broader
independently cached topology summary builders. Those require `nns-host` or the
complete `host` feature.

For the complete NNS governance, proposal, neuron, Registry inventory,
component-cache, and derived topology surface, use:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["nns-host"] }
```

`nns-host` is a strict superset of `nns-topology-host` and
`certified-subnet-catalog-host`. It exposes
`LiveNnsSource`, all NNS report-specific source traits and builders, Governance
proposal/neuron complete snapshots, component inventory caches, explicit
refresh policies, and progress events. Registry Prost decoding and SHA-256 are
direct dependencies through the topology/catalog substrate; CBOR is direct for
the authenticated Registry latest-version certificate. Dashboard Reqwest is
not a direct ic-query edge, although it remains transitive through `ic-agent`.
The certified version report does not promote ordinary Subnet Catalog
`get_value` reads beyond their separately recorded assurance. The
`fetch_nns_certified_registry_delta_batch_async` library operation validates
one caller-selected `get_certified_changes_since` batch on the caller's
runtime. It reports exact resource ceilings and `more_available` without
implicit pagination, caching, large-value retrieval, replay, or catalog
publication; custom async sources pass through the same pure
`validate_nns_certified_registry_delta_batch` structural contract. Custom
sources remain responsible for cryptographically authenticating the raw
certificate evidence they return.

To collect and retain the complete evidence sequence in one operation, use
`bootstrap_nns_certified_registry_archive_async` with an explicit
`NnsCertifiedRegistryArchiveBootstrapRequest`. The request combines the
source/time/replay bootstrap request with caller-selected cache and archive
roots, manifest/object ceilings, and lock staleness. It starts from Registry
version zero, reserves worst-case capacity before every call, holds one archive
refresh lock, locally reauthenticates every report, and publishes the final
manifest only after exact-target completion. Its custom-source counterpart
also performs built-in mainnet reauthentication; structural source validation
alone cannot establish archive authority. Neither function is called by a
load/read-through policy, and neither selects a default path or history size.

Archive manifest schema 1 supports explicit authenticated extension segments.
After a complete target, the next sealed batch selects a new segment target;
that target may equal the prior version, allowing a fresh empty delta to update
certificate-time evidence while preserving the same state digest. Use
`NnsCertifiedRegistryArchivePublisher::resume` to locally load, reauthenticate,
and consume an existing archive into a resumable publisher under explicit
cumulative replay and storage limits. Resume rewrites no historical object,
makes no source call, and acquires no refresh lock. Callers coordinating live
collection must hold the dedicated archive lock until `finish` publishes the
complete new segment. Manifests with another schema identifier have no
compatibility reader or migration and must be explicitly force-bootstrapped
again.

For the complete live boundary, call
`refresh_nns_certified_registry_archive_async` with an explicit
`NnsCertifiedRegistryArchiveRefreshRequest`. It requires an existing manifest,
rejects non-mainnet and missing archives before source work, and holds the
dedicated lock across resume, bounded collection, local authentication, and
atomic publication. The first response selects one exact successor target;
collection stops when that target completes, even if a later batch observes a
newer version. An unchanged version is published as a complete empty evidence
segment. The custom-source counterpart retains the same built-in mainnet
reauthentication boundary.

Interrupted publication can leave a durable content-addressed object that no
manifest references. Remove these only through the explicit local
`cleanup_nns_certified_registry_archive` operation. Its request supplies the
archive authentication limits, exact-directory scan ceiling, removal count and
byte ceilings, confined roots, observation time, and lock policy. Cleanup holds
the archive lock, reauthenticates the complete retained archive before
classification, scans no directory except the flat `objects/` directory, and
checks every ceiling before its first deletion. Its report returns the
authenticated archive plus exact scanned, referenced, removed, and removed-byte
counts. The operation is not an automatic load or refresh side effect.

Certified Subnet Catalog authority is available only through the complete
retained-evidence path. After publishing or loading an
`NnsAuthenticatedRegistryArchive`, call `project_nns_certified_subnet_catalog`
with an explicit `NnsCertifiedSubnetCatalogProjectionRequest`. The request owns
the `CatalogValidationContext` and requires a caller-selected maximum
certificate age plus `NnsCertifiedSubnetCatalogVersionPolicy`. Choose
`RequireLatestObserved` for current authority or `AllowHistoricalTarget` only
when an exact older Registry position is intentional. Excess age fails with
`StaleArchiveCertificate`; known version lag under require-latest fails with
`SupersededArchiveTarget`.

Successful authority exposes the exact observation time, certificate time,
nanosecond age, accepted maximum, selected and newest-observed versions, and
version policy through `.freshness()`. The returned
`NnsCertifiedSubnetCatalogAuthority` keeps its `ValidatedSubnetCatalog`
borrowed to that archive. Ordinary `ValidatedSubnetCatalog::try_from_raw`
continues to reject `CatalogAssurance::Certified`, so persisted JSON cannot
self-assert authority. Projection makes no network call and does not publish a
catalog cache.

To persist that projection, call `load_nns_certified_subnet_catalog` with one
`NnsCertifiedSubnetCatalogLoadRequest`. Its
`NnsCertifiedSubnetCatalogReadPolicy` selects cache-only,
publish-missing, publish-missing-or-invalid, or force-publication behavior.
The returned `NnsCertifiedSubnetCatalogLoadOutcome` keeps the archive-bound
authority attached and reports the exact cache path and disposition. All four
policies are local-only: they can publish from the supplied archive, but cannot
refresh it or make a network call. Recoverable invalidity is limited to cache
content; filesystem, projection, serialization, and accounting errors remain
failures.

For pure model/rendering use, keep all features off:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false }
```

No-default builds are checked for `wasm32-unknown-unknown` without `clap`,
`ic-agent`, Reqwest, Tokio, or `futures`. That is a host-dependency boundary,
not a `no_std` promise; the public DTOs may still use `String`, `Vec`, `serde`,
and other normal `std`-using crates.

## Progress Ownership

Normal report builders and refresh entry points do not write to stdout or
stderr. Paged NNS proposal/neuron and SNS neuron/proposal refreshes also expose
`*_with_progress` entry points. These accept a `QueryProgress` sink and emit
typed `QueryProgressEvent` values for cache creation and refresh lifecycle
updates. ICRC complete account-history refreshes expose the same pattern.
Observed node-status report builders accept the sink directly because an
ordinary read can visibly refresh one missing, invalid, or 60-second-stale
shared snapshot.
The sink supplied to that ICRC entry point must also be `Send` because its
synchronous host adapter may move the caller-owned sink across its internal
runtime boundary.

Downstream libraries can ignore progress by using the ordinary entry points,
record the events for their own UI, or render them at an executable boundary.
Terminal detection, same-line updates, and stderr output remain process policy
and are therefore implemented by `ic-query-cli`, not `ic-query`.

## Replace CLI Shell-Outs

A canic-style native crate should usually replace shell-outs in this order:

1. Build the matching public request type with its `new` constructor.
2. Call a report builder if the crate needs `host` behavior, or construct /
   deserialize a report DTO if it already has the data.
3. Consume the typed report directly for logic, or call the matching text
   renderer only at the display boundary.
4. Keep source endpoints explicit in the request so live network use remains
   visible.
5. Keep process arguments and output policy in the downstream executable;
   `ic-query` exposes no Clap or command-dispatch surface.

The CLI module layout is intentionally mirrored at the family level:

- `icq cache ...` maps to host-only `ic_query::cache`.
- `icq cloud-engine ...` maps to `ic_query::cloud_engine`.
- `icq ic ...` maps to `ic_query::ic`.
- Native `icq icrc ledger` and `account` operations map to
  `ic_query::icrc`; official REST-backed `icq icrc analytics` reports map to
  `ic_query::ic` so their Dashboard authority and live adapter stay explicit.
- `icq nns proposal ...` maps to `ic_query::nns::proposals`.
- `icq nns neuron ...` maps to `ic_query::nns::neuron`.
- `icq nns governance ...` maps to `ic_query::nns::governance`.
- `icq nns subnet ...` maps to `ic_query::subnet_catalog`.
- `icq nns node ...`, `data-center`, `node-provider`, and `node-operator` map
  to the matching `ic_query::nns::*` modules.
- Dashboard-backed `icq nns node|subnet|node-provider status` views map to
  `ic_query::ic`; their source authority and shared cache remain explicit even
  though the CLI noun accepts Registry-discovered principals.
- `icq nns topology ...` maps to `ic_query::nns::topology`.
- `icq sns ...` maps to `ic_query::sns`.
- `icq system ...` maps to `ic_query::system::cmc`.

These family roots are the only public paths. Internal `report` modules own
implementation details but are not available to downstream crates.

The library modules do not mirror every clap option type. They expose request
DTOs, report DTOs, builders, cache helpers, refresh helpers, and renderers.
SNS info, token, parameter, swap, upgrade, and Root-canister builders share
`SnsLookupRequest`; the bounded Governance metrics builder uses
`SnsMetricsRequest` because its proposal-count window is part of collection
intent. All read-only NNS topology builders share
`NnsTopologyReadRequest`; Registry-derived NNS
inventory families share the `NnsInventory*Request` contracts; SNS neuron and
proposal cache inspection shares the `SnsCache*` request and report contracts
plus `SnsRefreshAttemptStatus`; the joined discovery catalog uses
`SnsCatalogCacheRequest` and `SnsCatalogRefreshRequest`; complete NNS
Governance proposal and neuron
collections share `NnsGovernanceRefreshRequest`, `NnsGovernanceCacheRequest`,
and `NnsGovernanceRefreshAttemptStatus`; direct NNS Governance point-value
reports share `NnsSourceRequest` and one `NnsGovernanceSource` capability;
simple ledger-wide ICRC metadata and capability builders share
`IcrcLedgerRequest`. There are no per-report aliases for those canonical
types.
The examples below are covered by the `downstream_usage` integration test.

## Source Adapters

The public API exposes source adapters for host-only downstream crates that
need to reuse `ic-query` report assembly with data that does not come from the
built-in live adapters. CloudEngine, the official Dashboard, generic ICRC,
subnet catalog, NNS registry, NNS inventory, NNS proposal, NNS neuron, NNS
topology, SNS list/info/token/params/metrics/swap/upgrade/canister, SNS
proposal, and SNS neuron host APIs expose this pattern with
`CloudEngineSource`, `CloudEngineNodeSource`, `CloudEngineProviderSource`,
`IcCanisterSource`,
`IcCanisterCollectionSource`,
`IcMetricSource`, `IcNetworkSource`, `IcNodeStatusSource`,
`IcNodeProviderRewardSource`,
`IcIcrcAnalyticsSource`, and narrow native
ICRC capabilities such as
`IcrcTokenSource`,
`IcrcBalanceSource`, and `IcrcTransactionsSource`,
`build_icrc_*_report_with_source`,
`SubnetCatalogSource`, subnet catalog `*_with_source` builders,
`NnsRegistrySource`, the NNS inventory source traits, `NnsProposalSource`,
`NnsNeuronSource`, `NnsGovernanceSource`,
`NnsTopologySource`, `NnsTopologyRefreshSource`, `NnsSubnetTopologySource`,
`IcrcAccountTransactionPageSource`, `IcrcAccountTransactionCollectionSource`,
`SnsDiscoverySource`, `SnsCanisterSource`, `SnsTokenSource`, `SnsParamsSource`,
`SnsMetricsSource`, `SnsSwapSource`, `SnsUpgradeSource`,
`SnsProposalSource`, `SnsProposalsSource`, `SnsNeuronSource`,
`SnsNeuronsSource`, and `SnsRewardSource`.
`SnsDiscoverySource` keeps the authoritative SNS-W inventory separate from
explicit metadata targets. Direct report builders enrich exactly one resolved
SNS; `sns list` requests metadata for every inventory row. Custom sources must
return one canonical, unique metadata row for every requested Root principal
and no unrequested rows.
Certified Cycle Minting Canister reports expose `CmcSource` and the paired
`build_cmc_*_report_with_source` builders.
CloudEngine direct reports expose `CloudEngineSource` and the paired
operator/prices `*_with_source` builders. With both CloudEngine and Subnet
Catalog host features, the list report additionally exposes
`CloudEngineOperatorBindingSource` and
`build_cloud_engine_list_report_with_sources`; Registry catalog construction
and control-plane bindings retain separate source contracts.
Dashboard-backed provider reports expose `CloudEngineProviderSource` and the
paired provider `*_with_source` builders under `dashboard-host`. Pure provider
requests, reports, rows, locations, and text renderers remain available with
no host feature.
Dashboard-backed Type4 node reports expose `CloudEngineNodeSource` and paired
node list/info `*_with_source` builders under `dashboard-host`. Pure node
requests, reports, the shared raw row alias, filter constants, and text
renderers remain available with no host feature. Provider and node reports use
`DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT`; the earlier provider-named
constant was removed without an alias in 0.35.
Certified API boundary-node reporting exposes `IcApiBoundaryNodeSource` and
`build_ic_api_boundary_node_report_with_source`; its source contract carries
one authenticated complete state tree rather than Dashboard REST data.

The built-in implementations are deliberately less fragmented than the
capability traits. `ic_query::ic::LiveIcStateSource` owns certified IC state,
while `ic_query::ic::LiveIcSource` owns official Dashboard capabilities.
`ic_query::nns::LiveNnsSource` implements every supported NNS and
subnet-catalog source capability, while
`ic_query::sns::LiveSnsSource` and `ic_query::icrc::LiveIcrcSource` own their
respective live families. `ic_query::system::cmc::LiveCmcSource` owns the
focused CMC capability rather than adding one live adapter per report view.
`ic_query::cloud_engine::LiveCloudEngineSource` similarly owns the separate
control-plane authority.
NNS capabilities share
`ic_query::nns::NnsSourceRequest`; adding a new NNS report should normally add
a capability implementation to that adapter instead of introducing another
live-source type or another copy of the same provenance request. SNS
capabilities likewise share `SnsSourceRequest`, including explicit network and
collection provenance.

Use a custom source when a downstream tool needs to read from a mirror,
fixture, proxy, or pre-collected snapshot while still using `ic-query` report
assembly and text rendering:

```rust
use ic_query::nns::{
    NnsSourceRequest,
    registry::{
        NnsRegistryHostError, NnsRegistrySource, NnsRegistryVersionData,
        NnsRegistryVersionRequest, build_nns_registry_version_report_with_source,
        nns_registry_version_report_text,
    },
};

struct FixtureRegistrySource;

impl NnsRegistrySource for FixtureRegistrySource {
    fn fetch_registry_version(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsRegistryVersionData, NnsRegistryHostError> {
        Ok(NnsRegistryVersionData {
            network: "ic".to_string(),
            registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
            registry_version: 42,
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            source_endpoint: request.endpoint.clone(),
        })
    }
}

fn render_registry_version_with_source(
    source: &dyn NnsRegistrySource,
    now_unix_secs: u64,
) -> Result<String, NnsRegistryHostError> {
    let request = NnsRegistryVersionRequest::new(
        "ic",
        "https://mirror.example",
        now_unix_secs,
    );
    let report = build_nns_registry_version_report_with_source(&request, source)?;
    Ok(nns_registry_version_report_text(&report))
}
```

See [IC Reporting Adapters](design/ic-reporting-adapters.md) for the extension
rules and prioritized reporting backlog.

## Official Dashboard Examples

Native tools can build the same bounded official Dashboard report as
`icq ic canister info` without spawning the CLI:

```rust
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, IcCanisterRequest, IcHostError,
    build_ic_canister_report, ic_canister_report_text,
};

fn render_canister(
    canister_id: &str,
    now_unix_secs: u64,
) -> Result<String, IcHostError> {
    let request = IcCanisterRequest::new(
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
        now_unix_secs,
        canister_id,
    );
    let report = build_ic_canister_report(&request)?;
    Ok(ic_canister_report_text(&report))
}
```

This builder is host-only and always performs one live REST lookup. The report
identifies `official_ic_dashboard_api` as its authority and deliberately
states `certified: false` and `point_in_time_guaranteed: false`. It does not
read or write a cache, inherit a Registry version, or prove current
controller/module state.

Every built-in `LiveIcSource` lookup accepts at most
`MAX_IC_DASHBOARD_RESPONSE_BYTES` (8 MiB) of successful response body before
JSON decoding. Oversized declared or streamed responses return
`IcHostError::HttpResponseTooLarge`; incomplete bodies return
`IcHostError::HttpResponseBody`. Custom source capabilities provide already
decoded source data and therefore do not pass through this HTTP boundary.
Dashboard redirects are not followed; a 3xx response remains an
`IcHostError::HttpStatus` for the requested URL.

The shared live endpoint parser rejects credentials, queries, and fragments
for Dashboard and native agent base URLs. Every native `ic-agent` constructed
by the library also limits each response body to 8 MiB. These transport rules
do not change cache policy or turn targeted calls into collection operations.

No-default consumers can still construct and render `IcCanisterReport` values
without pulling in the live HTTP adapter.

Replica release discovery uses a separate focused source capability on the
same live adapter. It performs one page request, never follows the returned
offset automatically, and does not claim runtime-version evidence:

```rust
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, IcHostError,
    IcReplicaVersionListQuery, IcReplicaVersionListRequest,
    build_ic_replica_version_list_report,
};

fn recent_replica_versions(
    now_unix_secs: u64,
) -> Result<Vec<String>, IcHostError> {
    let request = IcReplicaVersionListRequest::new(
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
        now_unix_secs,
        IcReplicaVersionListQuery::new(25, 0, None),
    );
    let report = build_ic_replica_version_list_report(&request)?;
    Ok(report
        .rows
        .into_iter()
        .map(|row| row.replica_version_id)
        .collect())
}
```

The first response's `resolved_max_proposal_index` may be supplied in a later
query together with `report.next_offset`. Exact
`build_ic_replica_version_info_report` preserves the raw release summary and
Subnet rollout rows in one request. Both reports are live-only, off-chain,
uncertified, and separate from Registry desired-version or actual running
version evidence.

Node-provider rewards use another focused capability on `LiveIcSource`. One
explicit page retains the upstream overlap warning, while history remains a
single bounded aggregate request:

```rust
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, IcHostError,
    IcNodeProviderRewardHistoryQuery, IcNodeProviderRewardHistoryRequest,
    build_ic_node_provider_reward_history_report,
};

fn node_provider_reward_history(
    now_unix_secs: u64,
) -> Result<Vec<(u64, u64)>, IcHostError> {
    let request = IcNodeProviderRewardHistoryRequest::new(
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
        now_unix_secs,
        IcNodeProviderRewardHistoryQuery::new(
            now_unix_secs.saturating_sub(30 * 24 * 60 * 60),
            now_unix_secs,
            86_400,
        ),
    );
    let report = build_ic_node_provider_reward_history_report(&request)?;
    Ok(report
        .observations
        .into_iter()
        .map(|row| (row.timestamp_unix_secs, row.amount_e8s))
        .collect())
}
```

`build_ic_node_provider_reward_info_report` and
`build_ic_node_provider_reward_list_report` provide the exact and one-page
forms. Raw amounts remain e8s, timestamps remain Unix seconds, and
mode-specific detail stays a JSON object. Offset pages are not a complete
collection protocol and are never followed automatically.

Filtered discovery stays explicitly bounded. A count performs one REST request
and fetches no rows. A page performs one REST request for at most 100 rows and
does not follow the returned cursor or write a cache:

```rust
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT,
    IcCanisterFilters, IcCanisterPageRequest, IcHostError,
    build_ic_canister_page_report,
};

fn discover_named_canisters(
    now_unix_secs: u64,
) -> Result<Vec<String>, IcHostError> {
    let filters = IcCanisterFilters {
        has_name: Some(true),
        ..IcCanisterFilters::default()
    };
    let request = IcCanisterPageRequest::new(
        DEFAULT_IC_DASHBOARD_CANISTER_COLLECTION_SOURCE_ENDPOINT,
        now_unix_secs,
    )
    .with_filters(filters)
    .with_limit(25);
    let report = build_ic_canister_page_report(&request)?;
    Ok(report.rows.into_iter().map(|row| row.canister_id).collect())
}
```

Callers may supply `report.next_cursor` to `with_after` on a later request.
There is intentionally no automatic whole-catalog collection builder.

The same live adapter owns bounded network metrics through one focused source
capability. A request selects one official metric and an explicit window; it
cannot fan out over the metric catalog or create a cache:

```rust
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT, DEFAULT_IC_METRIC_STEP_SECS,
    IcHostError, IcMetricKind, IcMetricQuery, IcMetricRequest,
    build_ic_metric_report,
};

fn instruction_rate(
    start_unix_secs: u64,
    end_unix_secs: u64,
) -> Result<Vec<String>, IcHostError> {
    let query = IcMetricQuery::new(
        IcMetricKind::InstructionRate,
        start_unix_secs,
        end_unix_secs,
        DEFAULT_IC_METRIC_STEP_SECS,
    );
    let request = IcMetricRequest::new(
        DEFAULT_IC_DASHBOARD_METRICS_SOURCE_ENDPOINT,
        end_unix_secs,
        query,
    );
    let report = build_ic_metric_report(&request)?;
    Ok(report.series[0]
        .observations
        .iter()
        .map(|observation| observation.value.clone())
        .collect())
}
```

Metric values remain raw strings. The builder validates the official
time bounds and step, caps each requested series at 1,000 observations, and
records the same explicit non-certified Dashboard provenance as the canister
reports.

Finite network resources use a separate focused capability on the same live
adapter. Boundary-node rows are data-center aggregates, not individual nodes:

```rust
use ic_query::ic::{
    DEFAULT_IC_BOUNDARY_NODE_DATA_CENTERS_SOURCE_ENDPOINT,
    IcBoundaryNodeDataCentersRequest, IcHostError,
    build_ic_boundary_node_data_centers_report,
};

fn boundary_node_data_center_ids(
    now_unix_secs: u64,
) -> Result<Vec<String>, IcHostError> {
    let request = IcBoundaryNodeDataCentersRequest::new(
        DEFAULT_IC_BOUNDARY_NODE_DATA_CENTERS_SOURCE_ENDPOINT,
        now_unix_secs,
    );
    let report = build_ic_boundary_node_data_centers_report(&request)?;
    Ok(report.rows.into_iter().map(|row| row.dc_id).collect())
}
```

This builder makes one non-paginated REST request, preserves raw location and
count strings, includes zero-node rows, and never reads or writes a cache.

The same network capability also supports an explicitly bounded daily activity
window:

```rust
use ic_query::ic::{
    DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT, IcDailyStatsQuery,
    IcDailyStatsRequest, IcHostError, build_ic_daily_stats_report,
};

fn daily_average_transaction_rates(
    start_unix_secs: u64,
    end_unix_secs: u64,
) -> Result<Vec<String>, IcHostError> {
    let request = IcDailyStatsRequest::new(
        DEFAULT_IC_DASHBOARD_SOURCE_ENDPOINT,
        end_unix_secs,
        IcDailyStatsQuery::new(start_unix_secs, end_unix_secs),
    );
    let report = build_ic_daily_stats_report(&request)?;
    Ok(report
        .rows
        .into_iter()
        .map(|row| row.average_transactions_per_second)
        .collect())
}
```

Daily-statistics builders make one request, accept at most a 366-day window
and 366 rows, preserve selected rate values as raw strings, tolerate missing
days, and never read or write a cache.

Official ICRC analytics remain on `LiveIcSource` rather than the native
`LiveIcrcSource` because the values come from the Dashboard REST service. A
shared analytics request identifies one ledger, endpoint, and collection time;
total supply adds one explicit time window:

```rust
use ic_query::ic::{
    DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT, DEFAULT_ICRC_TOKEN_VALUE_LIMIT,
    DEFAULT_ICRC_TOKEN_VALUE_WINDOW_SECS, DEFAULT_ICRC_TOTAL_SUPPLY_STEP_SECS,
    DEFAULT_ICRC_TOTAL_SUPPLY_WINDOW_SECS, IcHostError, IcIcrcIndexedCountKind,
    IcIcrcIndexedCountRequest, IcIcrcTokenValueQuery, IcIcrcTokenValueRequest,
    IcIcrcTokenValueRow, IcIcrcTotalSupplyObservation, IcIcrcTotalSupplyQuery,
    IcIcrcTotalSupplyRequest, build_icrc_indexed_count_report, build_icrc_token_value_report,
    build_icrc_total_supply_report,
};

fn indexed_count(
    ledger_canister_id: &str,
    now_unix_secs: u64,
    kind: IcIcrcIndexedCountKind,
) -> Result<u64, IcHostError> {
    let request = IcIcrcIndexedCountRequest::new(
        DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT,
        now_unix_secs,
        ledger_canister_id,
        kind,
    );
    Ok(build_icrc_indexed_count_report(&request)?.total)
}

fn total_supply_history(
    ledger_canister_id: &str,
    now_unix_secs: u64,
) -> Result<Vec<IcIcrcTotalSupplyObservation>, IcHostError> {
    let request = IcIcrcTotalSupplyRequest::new(
        DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT,
        now_unix_secs,
        ledger_canister_id,
        IcIcrcTotalSupplyQuery::new(
            now_unix_secs.saturating_sub(DEFAULT_ICRC_TOTAL_SUPPLY_WINDOW_SECS),
            now_unix_secs,
            DEFAULT_ICRC_TOTAL_SUPPLY_STEP_SECS,
        ),
    );
    Ok(build_icrc_total_supply_report(&request)?.observations)
}

fn token_value_history(
    ledger_canister_id: &str,
    now_unix_secs: u64,
) -> Result<Vec<IcIcrcTokenValueRow>, IcHostError> {
    let request = IcIcrcTokenValueRequest::new(
        DEFAULT_ICRC_ANALYTICS_SOURCE_ENDPOINT,
        now_unix_secs,
        ledger_canister_id,
        IcIcrcTokenValueQuery::new(
            now_unix_secs.saturating_sub(DEFAULT_ICRC_TOKEN_VALUE_WINDOW_SECS),
            now_unix_secs,
            DEFAULT_ICRC_TOKEN_VALUE_LIMIT,
        ),
    );
    Ok(build_icrc_token_value_report(&request)?.rows)
}
```

Each builder makes exactly one request and never reads or writes a cache.
Account, holder, and transaction counts request no rows or cursors and retain
their exact typed kind in the report. Token values preserve nullable raw
external prices, 24-hour volumes, provider names, and URLs across a maximum
90-day/1,000-row request and expose possible limit truncation. Total supply
accepts only hourly or daily steps, caps the requested and returned series at
1,000 observations, and preserves raw base-unit strings. Their provenance is
explicitly off-chain and non-certified. `IcIcrcAnalyticsSource` lets host
consumers supply a fixture, mirror, or proxy through the same request and
result validation; no-default consumers can construct, serialize, and render
the query and report DTOs without enabling HTTP transport.

## Certified CMC Example

Native tools can query and verify the mainnet Cycle Minting Canister without
spawning the CLI:

```rust
use ic_query::system::cmc::{
    CmcHostError, CmcSourceRequest, DEFAULT_CMC_SOURCE_ENDPOINT,
    build_cmc_cycles_report,
};

fn cycles_per_icp(now_unix_secs: u64) -> Result<u128, CmcHostError> {
    let request = CmcSourceRequest::from_unix_secs(
        "ic",
        DEFAULT_CMC_SOURCE_ENDPOINT,
        now_unix_secs,
        "my-tool",
    );
    Ok(build_cmc_cycles_report(&request)?.cycles_per_icp)
}
```

The builder makes one `get_icp_xdr_conversion_rate` query. It authenticates
the certificate for the fixed mainnet CMC principal, validates the
certified-data hash-tree commitment, and proves the native rate leaf before
deriving cycles per ICP from the documented one-trillion-cycles-per-XDR
constant. Non-mainnet network identities are rejected before agent
construction. The operation is live-only and does not read or write a cache.

No-default consumers can construct, serialize, and render `CmcXdrReport` and
`CmcCyclesReport` without the live adapter. Host consumers can implement the
single `CmcSource` capability for a fixture or proxy and reuse the same report
projection.

## Pure Rendering Example

No-default consumers can use report DTOs and text renderers without native
live-call or CLI dependencies:

```rust
use ic_query::nns::registry::{
    NnsRegistryVersionReport, NnsRegistryVersionRequest,
    nns_registry_version_report_text,
};

fn render_registry_version() -> String {
    let request =
        NnsRegistryVersionRequest::new("ic", "https://icp-api.io", 1_700_000_000);

    let report = NnsRegistryVersionReport {
        schema_version: 1,
        network: request.network,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "my-tool".to_string(),
    };

    nns_registry_version_report_text(&report)
}
```

## Host Cache Example

Native tools can inventory every known complete cache locally through the
same generic report used by `icq cache status`:

```rust
use std::path::Path;

use ic_query::cache::{
    CacheStatusError, CacheStatusRequest, build_cache_status_report,
    cache_status_report_text,
};

fn render_cache_status(
    cache_root: &Path,
    now_unix_secs: u64,
) -> Result<String, CacheStatusError> {
    let request = CacheStatusRequest::new(cache_root, now_unix_secs);
    let report = build_cache_status_report(&request)?;
    Ok(cache_status_report_text(&report))
}
```

This inventory is bounded and local-only. It reports separate generic
`CacheHeaderStatus`, `CacheAgeStatus`, and `CacheRecoveryPolicy` evidence plus
self-described `CacheRefreshLockStatus`. The report records that it did not
perform family-specific semantic validation, does not scan large history
payloads, probe lock processes, or mutate local files, and leaves owning
family loaders authoritative for complete validation.

Native tools can use the same subnet catalog cache/report path as
`icq nns subnet info` without spawning `icq`:

```rust
use std::path::Path;

use ic_query::subnet_catalog::{
    DEFAULT_STALE_AFTER_SECONDS, DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT, ResolveAs,
    SubnetCatalogCacheRequest, SubnetCatalogHostError, SubnetCatalogInfoRequest,
    build_subnet_catalog_info_report, subnet_catalog_info_report_text,
};

fn render_subnet_info(
    cache_root: &Path,
    canister_or_subnet: &str,
    now_unix_secs: u64,
) -> Result<String, SubnetCatalogHostError> {
    let cache = SubnetCatalogCacheRequest::new(cache_root, "ic");
    let request = SubnetCatalogInfoRequest::new(
        cache,
        DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT,
        canister_or_subnet,
        now_unix_secs,
        DEFAULT_STALE_AFTER_SECONDS,
    )
    .with_forced(ResolveAs::Canister);

    let report = build_subnet_catalog_info_report(&request)?;
    Ok(subnet_catalog_info_report_text(&report))
}
```

`load_cached_subnet_catalog` accepts only a
`SubnetCatalogLoadRequest::cache_only` request. Network-capable callers use
`load_subnet_catalog` with an explicit `CatalogReadPolicy`; the returned
`CatalogLoadOutcome` contains a `ValidatedSubnetCatalog` and the exact
`CacheDisposition`. Report builders use the same policy and expose the
disposition in their reports. Current single-endpoint live collection is
always `CatalogAssurance::UncertifiedQuery`: the shared Registry version
prevents an internally skewed join but does not certify an ordinary query.
Callers can set `with_minimum_assurance` on the load request; evidence below
that threshold fails with `InsufficientAssurance` rather than being treated as
missing, invalid, or stale. The
`refresh_missing_invalid_or_older_than` constructor keeps the source and
maximum accepted age explicit.
Ordinary catalog caches use schema 1. Content with another schema identifier is
invalid and can be replaced only when the selected read policy explicitly
permits invalid cache refresh; there is no migration or fallback reader.

`ValidatedSubnetCatalog::resolve_canister_route` binds the canonical canister
and Subnet principals, complete matched `SubnetInfo`, routing range, Registry
version, binary catalog digest, and provenance in one result. The caller can
therefore use `SubnetKind` without a second catalog lookup.
`CatalogLoadOutcome::authority_evidence` returns a compact serializable record
of Registry version, digest, assurance, source endpoints, and cache
disposition for a durable plan. That record identifies the load outcome; it is
not a substitute for validated catalog content. The digest detects a payload
that was edited without being resealed; it is not a signature or local-tamper
boundary.
Async embedders can use `fetch_subnet_catalog_async`,
`load_subnet_catalog_async`, and `refresh_subnet_catalog_async` on their own
runtime. The async source seam returns `SubnetCatalogSourceFuture`; custom
sources must return single-endpoint evidence for the exact requested endpoint.
Dropping an in-flight async refresh releases its owned lock without publishing.

Agreement is an explicit bounded source selection rather than a different
cache or view:

```rust
use std::path::Path;

use ic_query::subnet_catalog::{
    CatalogSourceSelection, DEFAULT_REFRESH_LOCK_STALE_SECONDS,
    SubnetCatalogCacheRequest, SubnetCatalogHostError, SubnetCatalogRefreshReport,
    SubnetCatalogRefreshRequest, refresh_subnet_catalog_async,
};

async fn refresh_agreed_catalog(
    cache_root: &Path,
    endpoints: Vec<String>,
    now_unix_secs: u64,
) -> Result<SubnetCatalogRefreshReport, SubnetCatalogHostError> {
    let request = SubnetCatalogRefreshRequest::new(
        SubnetCatalogCacheRequest::new(cache_root, "ic"),
        CatalogSourceSelection::multi_endpoint_agreement(endpoints),
        now_unix_secs,
        DEFAULT_REFRESH_LOCK_STALE_SECONDS,
    );
    refresh_subnet_catalog_async(&request).await
}
```

Agreement accepts two or three distinct hostnames and succeeds only when all
sources return the same Registry version and canonical Registry payload. The
report records canonical endpoints, an agreement digest, and exact summed
Registry query-call counts. It remains non-certified evidence and never falls
back to one endpoint on mismatch.

## Exact-Version Subnet Topology

Placement-sensitive host tools should use the joined Subnet topology snapshot
instead of joining independently cached topology components. Its live source
resolves one Registry version and derives every Subnet, node, operator, and
provider relation at that exact version. The following API is available with
`features = ["nns-topology-host"]`, its `nns-host` superset, or the full `host`
feature:

```rust
use std::path::Path;

use ic_query::nns::topology::{
    DEFAULT_NNS_SUBNET_TOPOLOGY_REFRESH_LOCK_STALE_SECONDS,
    DEFAULT_NNS_SUBNET_TOPOLOGY_SOURCE_ENDPOINT, CachedNnsSubnetTopologyReport,
    NnsSubnetTopologyCacheRequest, NnsSubnetTopologyHostError,
    NnsSubnetTopologyRefreshRequest, refresh_nns_subnet_topology,
};

fn refresh_subnet_topology(
    cache_root: &Path,
    now_unix_secs: u64,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    let request = NnsSubnetTopologyRefreshRequest::new(
        NnsSubnetTopologyCacheRequest::new(cache_root, "ic"),
        DEFAULT_NNS_SUBNET_TOPOLOGY_SOURCE_ENDPOINT,
        now_unix_secs,
        DEFAULT_NNS_SUBNET_TOPOLOGY_REFRESH_LOCK_STALE_SECONDS,
    );
    refresh_nns_subnet_topology(&request)
}
```

Use `load_cached_nns_subnet_topology` for a strictly local read,
`load_or_refresh_missing_nns_subnet_topology` when only absence authorizes a
live call, and `load_or_refresh_stale_nns_subnet_topology` when a
caller-supplied age policy authorizes refresh. These operations are distinct
so a consumer cannot mistake read-through cache creation for freshness
enforcement.

## Complete ICRC Account History

Native consumers can collect and cache one complete account history without
shelling out to `icq`:

```rust
use std::path::Path;

use ic_query::icrc::{
    DEFAULT_ICRC_ACCOUNT_TRANSACTION_REFRESH_LOCK_STALE_SECONDS,
    IcrcAccountTransactionCacheRequest, IcrcAccountTransactionRefreshRequest,
    IcrcAccountTransactionRefreshReport, refresh_icrc_account_transaction_cache,
};

fn refresh_account_history(
    cache_root: &Path,
    now_unix_secs: u64,
    ledger_canister_id: &str,
    account_owner: &str,
) -> Result<IcrcAccountTransactionRefreshReport, ic_query::icrc::IcrcAccountTransactionError> {
    let cache = IcrcAccountTransactionCacheRequest::new(
        cache_root,
        "https://icp-api.io",
        ledger_canister_id,
        account_owner,
    );
    let request = IcrcAccountTransactionRefreshRequest::new(
        cache,
        now_unix_secs,
        100,
        DEFAULT_ICRC_ACCOUNT_TRANSACTION_REFRESH_LOCK_STALE_SECONDS,
    );
    refresh_icrc_account_transaction_cache(&request)
}
```

Use `load_cached_icrc_account_transactions` for cache-only access,
`load_or_refresh_missing_icrc_account_transactions` when absence authorizes a
live crawl, and `load_or_refresh_stale_icrc_account_transactions` only when a
caller-supplied age policy authorizes it. Complete account snapshots prove
index API exhaustion but carry `point_in_time_guaranteed: false`: the index
interface does not expose a snapshot version that can be held across pages.
Custom collection sources must return the explicitly requested index canister
when one is supplied. Failed refresh attempts retain the resolved index and
page/row/cursor evidence when collection reached that point.

## Native Report Example

NNS inventory modules expose cache/list/info request constructors, cache-backed
builders, refresh helpers, and renderers under `features = ["host"]`:

```rust
use std::path::Path;

use ic_query::nns::{
    NnsInventoryCacheRequest,
    node::{
        DEFAULT_NNS_NODE_SOURCE_ENDPOINT, NNS_NODE_SUBNET_KIND_APPLICATION,
        NnsNodeHostError, NnsNodeListRequest,
        build_nns_node_list_report, nns_node_list_report_text,
    },
};

fn render_application_nodes(
    cache_root: &Path,
    now_unix_secs: u64,
) -> Result<String, NnsNodeHostError> {
    let cache = NnsInventoryCacheRequest::new(cache_root, "ic");
    let request =
        NnsNodeListRequest::new(cache, DEFAULT_NNS_NODE_SOURCE_ENDPOINT, now_unix_secs)
            .with_subnet_kind(NNS_NODE_SUBNET_KIND_APPLICATION);

    let report = build_nns_node_list_report(&request)?;
    Ok(nns_node_list_report_text(&report))
}
```

## Live SNS Metrics Example

The bounded SNS Governance metrics builder uses two-request targeted discovery
and one metrics query. The window controls recent proposal counts; treasury
and voting-power values remain explicitly cached Governance evidence with
their own timestamps.

```rust
use ic_query::sns::{
    DEFAULT_SNS_SOURCE_ENDPOINT, SnsHostError, SnsMetricsRequest,
    build_sns_metrics_report, sns_metrics_report_text,
};

fn render_sns_metrics(
    sns_input: &str,
    now_unix_secs: u64,
) -> Result<String, SnsHostError> {
    let request = SnsMetricsRequest::new(
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        now_unix_secs,
        sns_input,
    )
    .with_time_window_seconds(90 * 24 * 60 * 60);

    let report = build_sns_metrics_report(&request)?;
    Ok(sns_metrics_report_text(&report))
}
```

## SNS Reward Evidence Example

Exact neuron detail and bracketed reward checkpoints are explicit live
capabilities under `host`; they do not expand the ordinary fixed-size neuron
cache. A checkpoint strictly exhausts native 100-row pages beneath the
Governance parameter ceiling and brackets them with complete parameter,
reward-event, and running-version responses.

```rust
use ic_query::sns::{
    DEFAULT_SNS_SOURCE_ENDPOINT, SnsHostError, SnsRewardCheckpointReport,
    SnsRewardCheckpointRequest, build_sns_reward_checkpoint_report,
};

fn collect_reward_checkpoint(
    sns_input: &str,
    now_unix_secs: u64,
) -> Result<SnsRewardCheckpointReport, SnsHostError> {
    let request = SnsRewardCheckpointRequest::new(
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        now_unix_secs,
        sns_input,
    );
    build_sns_reward_checkpoint_report(&request)
}
```

Pure checkpoint validation and reconciliation remain available with no
features. The diff builder treats both DTOs as untrusted, recomputes their raw
evidence, and returns typed invalid reasons instead of trusting serialized
totals or policy booleans:

```rust
use ic_query::sns::{
    SnsRewardCheckpointReport, SnsRewardCheckpointValidationError, SnsRewardDiffReport,
    build_sns_reward_diff_report, validate_sns_reward_checkpoint_report,
};

fn reconcile_reward(
    before: &SnsRewardCheckpointReport,
    after: &SnsRewardCheckpointReport,
) -> Result<SnsRewardDiffReport, SnsRewardCheckpointValidationError> {
    validate_sns_reward_checkpoint_report(before)?;
    validate_sns_reward_checkpoint_report(after)?;
    Ok(build_sns_reward_diff_report(before, after))
}
```

Native callers that want the library to load files can use
`build_sns_reward_diff_report_from_paths` under `host`. File selection and
persistence stay caller-owned; no checkpoint is implicitly written or cached,
and local checkpoint content is not authenticated.

## SNS Snapshot Example

Native tools can use SNS proposal and neuron snapshot APIs through `host`.
The joined deployed-SNS catalog also exposes cache-only,
refresh-if-one-hour-stale, and forced-refresh operations. Its cache identity is
the network-level collected catalog; list sorting and verbosity remain views.
Proposal list reports can create a missing complete proposal snapshot
through the public builder; whole-collection neuron sorts expect a prior
explicit refresh, matching the CLI cache policy. `SnsNeuronRow` preserves the
fixed-size native Governance values from the same `list_neurons` response,
including `SnsNeuronDissolveState`; variable permission and followee graphs
are not collected implicitly. The current neuron report and cache schema is 1;
any other snapshot shape is rejected and requires an explicit refresh.

```rust
use std::path::Path;

use ic_query::sns::{
    DEFAULT_SNS_SOURCE_ENDPOINT, SnsHostError, SnsNeuronsRequest, SnsNeuronsSort,
    SnsProposalSortDirection, SnsProposalsRequest, SnsProposalsSort,
    build_sns_neurons_report, build_sns_proposals_report, sns_neurons_report_text,
    sns_proposals_report_text,
};

fn render_recent_sns_proposals(
    cache_root: &Path,
    sns_input: &str,
    now_unix_secs: u64,
) -> Result<String, SnsHostError> {
    let request = SnsProposalsRequest::new(
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        now_unix_secs,
        sns_input,
        25,
    )
    .with_cache_root(cache_root)
    .with_sort(SnsProposalsSort::Created)
    .with_sort_direction(SnsProposalSortDirection::Desc);

    let report = build_sns_proposals_report(&request)?;
    Ok(sns_proposals_report_text(&report))
}

fn render_cached_sns_neurons(
    cache_root: &Path,
    sns_input: &str,
    now_unix_secs: u64,
) -> Result<String, SnsHostError> {
    let request = SnsNeuronsRequest::new(
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        now_unix_secs,
        sns_input,
        500,
    )
    .with_cache_root(cache_root)
    .with_sort(SnsNeuronsSort::Stake);

    let report = build_sns_neurons_report(&request)?;
    Ok(sns_neurons_report_text(&report))
}
```

Local cache inspection remains available without making live calls:

```rust
use std::path::Path;

use ic_query::sns::{
    SnsCacheStatusRequest, SnsHostError,
    build_sns_neurons_cache_status_report, build_sns_proposals_cache_status_report,
    sns_neurons_cache_status_report_text, sns_proposals_cache_status_report_text,
};

fn render_sns_cache_status(cache_root: &Path, sns_input: &str) -> Result<String, SnsHostError> {
    let proposals = SnsCacheStatusRequest::new(cache_root, "ic", sns_input);
    let proposals_report = build_sns_proposals_cache_status_report(&proposals)?;

    let neurons = SnsCacheStatusRequest::new(cache_root, "ic", sns_input);
    let neurons_report = build_sns_neurons_cache_status_report(&neurons)?;

    Ok(format!(
        "{}\n{}",
        sns_proposals_cache_status_report_text(&proposals_report),
        sns_neurons_cache_status_report_text(&neurons_report)
    ))
}
```

## Live ICRC Example

Generic ICRC builders are live-only and keep the queried endpoint explicit:

```rust
use ic_query::icrc::{
    DEFAULT_ICRC_SOURCE_ENDPOINT, IcrcError, IcrcLedgerRequest,
    build_icrc_token_report, icrc_token_report_text,
};

fn render_token(
    ledger_canister_id: &str,
    now_unix_secs: u64,
) -> Result<String, IcrcError> {
    let request =
        IcrcLedgerRequest::new(DEFAULT_ICRC_SOURCE_ENDPOINT, now_unix_secs, ledger_canister_id);
    let report = build_icrc_token_report(&request)?;
    Ok(icrc_token_report_text(&report))
}
```

## When Not To Use It

Do not route every simple public query through `ic-query` automatically. If a
frontend can cheaply query a public canister directly, and it does not need
`ic-query` report shaping, snapshot/cache semantics, joins, or shared text/JSON
rendering, a direct frontend query can be simpler and cheaper.

`ic-query` is most useful when a downstream crate wants one of these:

- A typed report model shared with `icq`.
- Cache-backed NNS/SNS inventory or topology behavior.
- Complete snapshot refresh and local inspection.
- Reusable text/JSON rendering that matches the CLI.
- A native Rust boundary instead of a child-process boundary.
