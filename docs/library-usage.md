# Library Usage

This guide is for Rust crates that want to call `ic-query` directly instead of
spawning the `icq` executable.

The usual downstream shape is:

```toml
[dependencies]
ic-query = { version = "0.29", default-features = false, features = ["host"] }
```

Use `host` for native tools that need live calls, filesystem caches, refresh
operations, or cache-backed report builders. The library has no CLI feature;
`icq` parsing and dispatch are owned by `ic-query-cli`.

For a native embedder that needs only the live/cache Subnet catalog API, use
the narrower feature:

```toml
[dependencies]
ic-query = { version = "0.29", default-features = false, features = ["subnet-catalog-host"] }
```

`subnet-catalog-host` includes the IC agent, Registry protobuf decoding,
hashing, synchronous Tokio bridge, capability-rooted cache IO through
`cap-std`/`cap-fs-ext`, and endpoint validation required by
`ic_query::subnet_catalog`. It does not enable `ic-query`'s direct optional
Dashboard `reqwest` transport or `serde_cbor` certification dependencies. Those
packages may still appear transitively through `ic-agent`. The full `host`
feature is a strict superset.

For pure model/rendering use, keep all features off:

```toml
[dependencies]
ic-query = { version = "0.29", default-features = false }
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
built-in live adapters. The official Dashboard, generic ICRC, subnet catalog,
NNS registry, NNS inventory, NNS proposal, NNS neuron, NNS topology, SNS
list/info/token/params/metrics/swap/upgrade/canister, SNS proposal, and SNS
neuron host APIs expose
this pattern with `IcCanisterSource`, `IcCanisterCollectionSource`,
`IcMetricSource`, `IcNetworkSource`, `IcNodeStatusSource`,
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

The built-in implementations are deliberately less fragmented than the
capability traits. `ic_query::ic::LiveIcSource` owns official Dashboard
capabilities, `ic_query::nns::LiveNnsSource` implements every supported NNS and
subnet-catalog source capability, while
`ic_query::sns::LiveSnsSource` and `ic_query::icrc::LiveIcrcSource` own their
respective live families. `ic_query::system::cmc::LiveCmcSource` owns the
focused CMC capability rather than adding one live adapter per report view.
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

`ValidatedSubnetCatalog::resolve_canister_route` binds the canonical canister
and Subnet principals, matched routing range, Registry version, binary catalog
digest, and provenance in one result. The digest detects a payload that was
edited without being resealed; it is not a signature or local-tamper boundary.
Async embedders can use `fetch_subnet_catalog_async` on their own runtime.

## Exact-Version Subnet Topology

Placement-sensitive host tools should use the joined Subnet topology snapshot
instead of joining independently cached topology components. Its live source
resolves one Registry version and derives every Subnet, node, operator, and
provider relation at that exact version:

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
