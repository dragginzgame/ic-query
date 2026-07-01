# Library Usage

This guide is for Rust crates that want to call `ic-query` directly instead of
spawning the `icq` executable.

The usual downstream shape is:

```toml
[dependencies]
ic-query = { version = "0.6", default-features = false, features = ["host"] }
```

Use `host` for native tools that need live calls, filesystem caches, refresh
commands, or cache-backed report builders. Do not enable `cli` unless the crate
really wants `icq` command parsing and dispatch.

For pure model/rendering use, keep all features off:

```toml
[dependencies]
ic-query = { version = "0.6", default-features = false }
```

No-default builds are checked for `wasm32-unknown-unknown` without `clap`,
`ic-agent`, Tokio, or `futures`. That is a host-dependency boundary, not a
`no_std` promise; the public DTOs may still use `String`, `Vec`, `serde`, and
other normal `std`-using crates.

## Replace CLI Shell-Outs

A canic-style native crate should usually replace shell-outs in this order:

1. Build the matching public request type with its `new` constructor.
2. Call a report builder if the crate needs `host` behavior, or construct /
   deserialize a report DTO if it already has the data.
3. Consume the typed report directly for logic, or call the matching text
   renderer only at the display boundary.
4. Keep source endpoints explicit in the request so live network use remains
   visible.
5. Avoid the `cli` feature unless the downstream crate is intentionally
   wrapping `icq` command parsing.

The CLI module layout is intentionally mirrored at the family level:

- `icq icrc ...` maps to `ic_query::icrc`.
- `icq nns proposal ...` maps to `ic_query::nns::proposals`.
- `icq nns subnet ...` maps to `ic_query::subnet_catalog`.
- `icq nns node ...`, `data-center`, `node-provider`, and `node-operator` map
  to the matching `ic_query::nns::*` modules.
- `icq nns topology ...` maps to `ic_query::nns::topology`.
- `icq sns ...` maps to `ic_query::sns`.

The library modules do not mirror every clap option type. They expose request
DTOs, report DTOs, builders, cache helpers, refresh helpers, and renderers.
The examples below are covered by the `downstream_usage` integration test.

## 0.5 Source Boundary

The 0.5 public API uses the built-in host source adapters behind public report
builders. Source traits used by the report internals and fixture tests are not
public or stable in this release line. Downstream crates that need custom
canister or fixture sources should use public request/report DTOs at their own
boundary for now; a public source-trait adapter design belongs in a later
minor release.

In 0.6, the generic ICRC, subnet catalog, NNS registry, and NNS inventory host
APIs start that public source-adapter work with `IcrcSource`,
`build_icrc_*_report_with_source`, `SubnetCatalogSource`, subnet catalog
`*_with_source` builders, `NnsRegistrySource`, and the NNS node,
data-center, node-provider, and node-operator source traits and
`*_with_source` builders. NNS proposal/topology and SNS source traits remain
internal until their family-specific source DTOs are reviewed as public
contracts.

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
    project_root: &Path,
    canister_or_subnet: &str,
    now_unix_secs: u64,
) -> Result<String, SubnetCatalogHostError> {
    let cache = SubnetCatalogCacheRequest::new(project_root, "ic");
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

`load_cached_subnet_catalog` is cache-only. `load_or_refresh_subnet_catalog`
and the report builders can refresh missing cache data, so downstream commands
should surface that behavior clearly.

## Native Report Example

NNS inventory modules expose cache/list/info request constructors, cache-backed
builders, refresh helpers, and renderers under `features = ["host"]`:

```rust
use std::path::Path;

use ic_query::nns::node::{
    DEFAULT_NNS_NODE_SOURCE_ENDPOINT, NNS_NODE_SUBNET_KIND_APPLICATION,
    NnsNodeCacheRequest, NnsNodeHostError, NnsNodeListRequest,
    build_nns_node_list_report, nns_node_list_report_text,
};

fn render_application_nodes(
    project_root: &Path,
    now_unix_secs: u64,
) -> Result<String, NnsNodeHostError> {
    let cache = NnsNodeCacheRequest::new(project_root, "ic");
    let request =
        NnsNodeListRequest::new(cache, DEFAULT_NNS_NODE_SOURCE_ENDPOINT, now_unix_secs)
            .with_subnet_kind(NNS_NODE_SUBNET_KIND_APPLICATION);

    let report = build_nns_node_list_report(&request)?;
    Ok(nns_node_list_report_text(&report))
}
```

## SNS Snapshot Example

Native tools can use SNS proposal and neuron snapshot APIs without enabling
`cli`. Proposal list reports can create a missing complete proposal snapshot
through the public builder; whole-collection neuron sorts expect a prior
explicit refresh, matching the CLI cache policy.

```rust
use std::path::Path;

use ic_query::sns::{
    DEFAULT_SNS_SOURCE_ENDPOINT, SnsHostError, SnsNeuronsRequest, SnsNeuronsSort,
    SnsProposalSortDirection, SnsProposalsRequest, SnsProposalsSort,
    build_sns_neurons_report, build_sns_proposals_report, sns_neurons_report_text,
    sns_proposals_report_text,
};

fn render_recent_sns_proposals(
    project_root: &Path,
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
    .with_icp_root(project_root)
    .with_sort(SnsProposalsSort::Created)
    .with_sort_direction(SnsProposalSortDirection::Desc);

    let report = build_sns_proposals_report(&request)?;
    Ok(sns_proposals_report_text(&report))
}

fn render_cached_sns_neurons(
    project_root: &Path,
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
    .with_icp_root(project_root)
    .with_sort(SnsNeuronsSort::Stake);

    let report = build_sns_neurons_report(&request)?;
    Ok(sns_neurons_report_text(&report))
}
```

Local cache inspection remains available without making live calls:

```rust
use std::path::Path;

use ic_query::sns::{
    SnsHostError, SnsNeuronsCacheStatusRequest, SnsProposalsCacheStatusRequest,
    build_sns_neurons_cache_status_report, build_sns_proposals_cache_status_report,
    sns_neurons_cache_status_report_text, sns_proposals_cache_status_report_text,
};

fn render_sns_cache_status(project_root: &Path, sns_input: &str) -> Result<String, SnsHostError> {
    let proposals = SnsProposalsCacheStatusRequest::new(project_root, "ic", sns_input);
    let proposals_report = build_sns_proposals_cache_status_report(&proposals)?;

    let neurons = SnsNeuronsCacheStatusRequest::new(project_root, "ic", sns_input);
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
    DEFAULT_ICRC_SOURCE_ENDPOINT, IcrcError, IcrcTokenRequest,
    build_icrc_token_report, icrc_token_report_text,
};

fn render_token(
    ledger_canister_id: &str,
    now_unix_secs: u64,
) -> Result<String, IcrcError> {
    let request =
        IcrcTokenRequest::new(DEFAULT_ICRC_SOURCE_ENDPOINT, now_unix_secs, ledger_canister_id);
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
