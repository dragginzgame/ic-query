# Rust API reference

[Quick start](../README.md) · [Library examples](library-usage.md) · [Command and evidence reference](command-reference.md)

Use `ic-query` for typed requests, reports, validation, cache behavior, source
adapters, and renderers without spawning `icq`.

The pending [0.86 consumer update](changelog/0.86.md) covers canonical catalog
load failures, typed report origins and SNS lifecycle outcomes, account-history
contexts, and grouped reward-diff observations. Continuation status is derived
from admitted page/cursor facts; all schemas remain `1`.

Pure DTO and rendering use has no host dependencies:

```toml
[dependencies]
ic-query = { version = "0.89", default-features = false }
```

Native tools that need live calls, filesystem caches, refreshes, or custom
source adapters enable `host`:

```toml
[dependencies]
ic-query = { version = "0.89", default-features = false, features = ["host"] }
```

The no-default build is checked for `wasm32-unknown-unknown` without Clap,
`ic-agent`, Reqwest, Tokio, or `futures-util`. This is a host-dependency boundary,
not a `no_std` promise.

Rust canisters can enable the focused IC runtime adapter without pulling in
the native host graph:

```toml
[dependencies]
ic-query = { version = "0.89", default-features = false, features = ["canister"] }
```

The canister surface collects the four bounded direct NNS Governance point
reports plus proposal and neuron pages and exact proposal or neuron detail
through replicated inter-canister calls. Proposal and neuron pages can also be
advanced through serializable caller-owned continuations:

```rust,no_run
use ic_query::nns::governance::{
    CanisterNnsSource, NnsGovernanceError, NnsGovernanceMetricsReport,
    NnsGovernanceRequest, build_nns_governance_metrics_report_with_source,
};

async fn governance_metrics(
    now_unix_secs: u64,
) -> Result<NnsGovernanceMetricsReport, NnsGovernanceError> {
    let request = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs(
        "ic",
        now_unix_secs,
    );
    build_nns_governance_metrics_report_with_source(&request, &CanisterNnsSource).await
}
```

Proposal collection uses the same request, provenance, and source boundary. A
list accepts 1 through 100 rows and never follows the next cursor:

```rust,no_run
use ic_query::nns::{
    governance::{CanisterNnsSource, NnsGovernanceRequest},
    proposals::{
        NnsProposalError, NnsProposalListReport, NnsProposalListRequest,
        build_nns_proposal_list_report_with_source,
    },
};

async fn latest_proposals(
    now_unix_secs: u64,
) -> Result<NnsProposalListReport, NnsProposalError> {
    let governance = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs(
        "ic",
        now_unix_secs,
    );
    let request = NnsProposalListRequest::new(governance, 25);
    build_nns_proposal_list_report_with_source(&request, &CanisterNnsSource).await
}
```

For a complete proposal or public-neuron walk, create the corresponding
`NnsProposalCollectionState` or `NnsNeuronCollectionState` with an explicit
page size and cumulative page ceiling, then persist each returned page before
replacing the retained state. Every advance makes exactly one call. `complete`
means API exhaustion; `page_limit_reached` preserves bounded progress without
making that claim. The states define no stable-memory layout or publication
policy, and sequential pages are not one point-in-time view.

Once a proposal walk is complete, its retained rows can be projected locally
without another source call or a runtime-specific implementation:

```rust,no_run
use ic_query::nns::proposals::{
    NnsProposalActivityError, NnsProposalActivityReport,
    NnsProposalActivityRequest, NnsProposalCollectionState, NnsProposalRow,
    build_nns_proposal_activity_report,
};

fn proposal_activity(
    state: &NnsProposalCollectionState,
    proposals: &[NnsProposalRow],
) -> Result<NnsProposalActivityReport, NnsProposalActivityError> {
    build_nns_proposal_activity_report(
        &NnsProposalActivityRequest {
            from_proposal_timestamp_seconds: None,
            until_proposal_timestamp_seconds: None,
        },
        state,
        proposals,
    )
}
```

The schema-1 report preserves raw topic, decision-status, and reward-status
codes, provides UTC-day counts, and records that a sequential collection is
not one point-in-time snapshot. The builder rejects partial collections,
row-count mismatches, duplicate or missing proposal ids, zero timestamps, and
raw/typed classification inconsistencies before returning a report. It is
available in the no-default build as well as to callers using `canister` or a
native host feature. After restoring a report from caller-owned storage, use
`validate_nns_proposal_activity_report` to recheck its schema-1 identity,
provenance, window accounting, included range, and canonical count tables
without making a source call.

One neuron list call accepts 1 through 300 rows, returns the validated next
neuron-id cursor when the page is full, and never follows it automatically.
Exact detail makes one `get_neuron_info` call. Complete filesystem cache
refresh remains native-only, but it uses the same portable continuation
engine.

After that walk reaches API exhaustion, its retained public rows can be
projected into a portable distribution without another source call:

```rust,no_run
use ic_query::nns::neuron::{
    NnsNeuronCollectionState, NnsNeuronDistributionError,
    NnsNeuronDistributionReport, NnsNeuronRow,
    build_nns_neuron_distribution_report,
};

fn neuron_distribution(
    state: &NnsNeuronCollectionState,
    neurons: &[NnsNeuronRow],
) -> Result<NnsNeuronDistributionReport, NnsNeuronDistributionError> {
    build_nns_neuron_distribution_report(state, neurons)
}
```

The schema-1 report preserves raw state, optional visibility, optional neuron-
type, effective-stake, optional-field coverage, source, and retrieval-time
evidence. It explicitly does not turn the limited unauthenticated public rows
or their sequential collection into owner state or a point-in-time balance.
Use `validate_nns_neuron_distribution_report` after restoring a caller-owned
serialized report.

Call these builders from an update, timer, heartbeat, or another replicated
execution context that permits inter-canister calls. Each builder or
collection advance issues one bounded-wait call, attaches no cycles, performs
no retry or state write, and records the executing collector canister in
tagged provenance. The caller owns scheduling, retry policy, cycle budgeting,
and persistence. The `canister` feature does not enable `host`, `ic-agent`,
Tokio, Reqwest, Futures, or the filesystem cache graph.

Focused host features let embedders select one reporting family:

| Feature | Host surface |
| --- | --- |
| `cloud-engine-host` | Public CloudEngine operator-binding, operator-detail, and marketplace adapters; combine with `subnet-catalog-host` for the list builder |
| `cmc-host` | Certified Cycle Minting Canister ICP/XDR and cycles reports |
| `dashboard-host` | Official Dashboard REST reports, node-provider reward collection, CloudEngine provider and Type4 node collection, and observed default-scope node-status cache |
| `ic-state-host` | Certified API boundary-node state-tree collection |
| `icrc-host` | Native ICRC ledger/index reports and complete account-history cache |
| `nns-host` | Complete NNS governance, certified Registry latest-version reports, Registry inventory, component-cache, and topology APIs |
| `nns-topology-host` | Exact-version joined NNS topology plus Subnet Catalog |
| `sns-host` | Native SNS reports, caches, and reward evidence |
| `subnet-catalog-host` | Focused Subnet Catalog authority, cache, and resolution APIs |
| `host` | Convenience union of every focused host family |
| `certificate-host` | Bounded typed certificate/witness authentication and certified ledger tips |
| `ledger-evidence-host` | Certificate support plus immutable bounded generic block-chain verification |

Public report families are exposed from:

- `ic_query::cache` for shared models, with inventory builders and rendering
  under the `host` feature
- `ic_query::cloud_engine`
- `ic_query::ic`
- `ic_query::icrc`
- `ic_query::nns`
- `ic_query::sns`
- `ic_query::subnet_catalog`
- `ic_query::system::cmc`

Built-in host calls use one adapter per authority family:
`LiveCloudEngineSource`, `LiveIcStateSource`, `LiveIcSource`,
`LiveIcrcSource`, `LiveNnsSource`, `LiveSnsSource`, and `LiveCmcSource`.
Report-specific capability traits let fixtures, mirrors, proxies, and
pre-collected sources reuse the same validation and projection path.

Live collection and read-through builders are async and use the caller's Tokio
runtime. Pure projections, renderers and explicit cache-only reads remain
synchronous. Construct stateful live adapters with `::default()` and reuse one
per collection; their validated transports retain the selected endpoint and
response limits. Custom host sources require `Sync`, source futures are `Send`,
and progress sinks require `Send`.

Library builders do not write to stdout or stderr. Paged refresh APIs can emit
typed `QueryProgressEvent` values to a caller-provided sink; terminal rendering
remains an `ic-query-cli` responsibility.

Enable `cloud-engine-host` when an embedder needs only the public CloudEngine
operator and marketplace reports:

```toml
[dependencies]
ic-query = { version = "0.89", default-features = false, features = ["cloud-engine-host"] }
```

This enables `ic-agent`, Tokio, and URL validation without ic-query's direct
cache-filesystem, Registry Prost, Futures, Reqwest, CBOR, or SHA-256 edges.
Reqwest, CBOR, and cryptographic packages can remain transitive through
`ic-agent`.

The Registry-backed CloudEngine list builder requires both focused authority
features (or the convenience `host` feature):

```toml
[dependencies]
ic-query = { version = "0.89", default-features = false, features = ["cloud-engine-host", "subnet-catalog-host"] }
```

Enable `dashboard-host` when an embedder needs only the official Dashboard
REST reports, node-provider reward collection, CloudEngine provider and Type4
node collection, and the shared observed default-scope node-status cache:

```toml
[dependencies]
ic-query = { version = "0.89", default-features = false, features = ["dashboard-host"] }
```

This exposes `LiveIcSource`, its custom-source traits and builders including
`IcNodeProviderRewardSource`, `CloudEngineNodeSource`, and
`CloudEngineProviderSource`, and the confined node-status cache without
enabling `ic-agent`, Registry protobufs,
native NNS/SNS/ICRC host adapters, or CBOR certification. Reqwest may retain
cryptographic packages such as SHA-256 implementations transitively; the
feature promises the absence of ic-query's direct Registry and certification
edges, not every similarly named transitive package.

Enable `ic-state-host` for only the certified API boundary-node state-tree
report:

```toml
[dependencies]
ic-query = { version = "0.89", default-features = false, features = ["ic-state-host"] }
```

This exposes `LiveIcStateSource`, its focused source trait, and live/custom
builders. It enables `ic-agent`, Tokio, and URL validation without the cache
filesystem, Dashboard Reqwest transport, Registry Prost decoding, Futures,
direct CBOR decoding, or direct SHA-256 hashing.

Enable `cmc-host` when an embedder needs only authenticated Cycle Minting
Canister ICP/XDR and cycles reports:

```toml
[dependencies]
ic-query = { version = "0.89", default-features = false, features = ["cmc-host"] }
```

This enables `ic-agent` and direct CBOR certificate/witness decoding without
the cache filesystem or Registry protobuf graph. It has no direct ic-query
Futures, Reqwest, Prost, or SHA-256 edge; Reqwest and cryptographic packages
remain transitive through `ic-agent`.

Enable `icrc-host` for native ICRC ledger/index queries, certified-tip
verification, and the complete account-history cache:

```toml
[dependencies]
ic-query = { version = "0.89", default-features = false, features = ["icrc-host"] }
```

This leaves Dashboard, Registry, NNS, and SNS host adapters disabled and does
not enable ic-query's direct Reqwest or Prost dependencies. `ic-agent` retains
Reqwest transitively, so the package may still appear in the complete graph.

Enable `sns-host` for native SNS discovery, targeted reports, complete
proposal/neuron caches:

```toml
[dependencies]
ic-query = { version = "0.89", default-features = false, features = ["sns-host"] }
```

This leaves Dashboard, Registry, NNS, system-canister, and native ICRC host
adapters disabled. It shares the ICRC token-metadata mechanics required to
inspect SNS ledgers, but does not enable the native ICRC report or cache
surface. There is no direct ic-query edge to Reqwest, Prost, CBOR, or SHA-256;
`ic-agent` retains Reqwest, CBOR, and cryptographic packages transitively.

Enable the narrower `subnet-catalog-host` feature when a native embedder needs
only live/cache Subnet catalog behavior. It keeps the IC agent, Registry
decoding, capability-filesystem dependencies, and other cache
dependencies required by that API while leaving `ic-query`'s direct optional
Dashboard `reqwest` transport disabled. Certified Registry calls additionally
select Host's bounded certificate decoder; direct `serde_cbor` use is fixture-only.
Because the feature still includes `ic-agent`, Reqwest and CBOR remain in its
transitive dependency graph. The full `host` feature remains the
choice for all reporting adapters and is a strict superset.

Standard live Subnet Catalog load and refresh calls reuse Registry history
across processes under the request's private cache root. Embedders supplying
their own `LiveSubnetCatalogSource` enable the same behavior with
`with_history_cache(cache_root)`. Cache-only reads perform no history IO or
network calls; dry-run convenience refreshes use memory-only history. The
bounded schema-1 transcript retains local query evidence; every live
acquisition still reads current pinned records and checks the
requested endpoint agreement. See the
[history reuse contract](../docs/design/subnet-catalog-acquisition-performance.md#cross-process-transcript-contract)
for ownership, limits, progress, and recovery policy.

Enable `nns-topology-host` when an embedder also needs the exact-version joined
NNS Subnet/node/operator/provider topology cache and source API:

```toml
[dependencies]
ic-query = { version = "0.89", default-features = false, features = ["nns-topology-host"] }
```

This feature includes `subnet-catalog-host` but not ic-query's direct optional
Dashboard Reqwest or CBOR certification edges. It does not expose the broader
component-cache topology summary builders; those require `nns-host` or the
complete `host` feature.

Enable `nns-host` for the complete NNS governance, proposal, neuron, Registry
inventory, component-cache, and derived topology host API:

```toml
[dependencies]
ic-query = { version = "0.89", default-features = false, features = ["nns-host"] }
```

This includes `nns-topology-host` and directly includes the Registry Prost,
SHA-256, and CBOR dependencies needed by exact-version topology, endpoint
agreement, and certified Registry evidence. It has no direct ic-query
Dashboard Reqwest edge; Reqwest remains transitive through `ic-agent`.

The Registry latest-version builder authenticates its certificate and committed
version leaf. Registry archive/delta replay and archive-backed catalog authority
are retired. Preserve existing evidence files with the former reader; the current
library does not load, migrate or delete them. Ordinary Registry history transcripts
remain local query evidence and do not establish certified catalog assurance.

Complete Governance/ICRC cache refreshes, Subnet Catalog report builders and the
CloudEngine list builder retain a live entry point and one explicit custom-source
plus progress entry point. Use the matching live adapter with a progress sink, or
pass a no-op closure to the source/progress function. Intermediate source-only and
progress-only overloads for these operations have been removed.

The Subnet Catalog API separates serde-facing `RawSubnetCatalog` data from
private-field `ValidatedSubnetCatalog` evidence. Explicit load policies return
both the validated catalog and observable acquisition path/disposition.
`CatalogLoadOutcome::snapshot_authority()` derives a stable persistable
`CatalogSnapshotAuthorityEvidence` only from the validated catalog; cache hits
and refresh dispositions are deliberately excluded. Validated canister
resolution returns the matched range, Registry version, catalog digest, and
full provenance together. Single-endpoint live collection is always labelled
`CatalogAssurance::UncertifiedQuery`. Async embedders can call
`fetch_subnet_catalog_async`, `load_subnet_catalog`, or
`refresh_subnet_catalog` on their own runtime. Dropping an async refresh
releases its owned lock without publishing. Live operations return futures
polled by the caller; the library does not create runtimes or helper threads.
Only schema-1 catalog files are accepted; an explicit read policy may refresh
unsupported ordinary cache content, but no migration or legacy reader is
retained.

See [Library Usage](https://github.com/dragginzgame/ic-query/blob/main/docs/library-usage.md) for complete examples and feature
guidance.
