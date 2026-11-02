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
ic-query = { version = "0.62", default-features = false }
```

Native tools that need live calls, filesystem caches, refreshes, or custom
source adapters enable `host`:

```toml
[dependencies]
ic-query = { version = "0.62", default-features = false, features = ["host"] }
```

The no-default build is checked for `wasm32-unknown-unknown` without Clap,
`ic-agent`, Reqwest, Tokio, or `futures-util`. This is a host-dependency boundary,
not a `no_std` promise.

Rust canisters can enable the focused IC runtime adapter without pulling in
the native host graph:

```toml
[dependencies]
ic-query = { version = "0.62", default-features = false, features = ["canister"] }
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
| `certified-subnet-catalog-host` | Subnet Catalog plus certified Registry batches, archive/replay, and archive-bound catalog authority |
| `dashboard-host` | Official Dashboard REST reports, node-provider reward collection, CloudEngine provider and Type4 node collection, and observed default-scope node-status cache |
| `ic-state-host` | Certified API boundary-node state-tree collection |
| `icrc-host` | Native ICRC ledger/index reports and complete account-history cache |
| `nns-host` | Complete NNS governance, certified Registry evidence and pure replay, Registry inventory, component-cache, and topology APIs |
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
ic-query = { version = "0.62", default-features = false, features = ["cloud-engine-host"] }
```

This enables `ic-agent`, Tokio, and URL validation without ic-query's direct
cache-filesystem, Registry Prost, Futures, Reqwest, CBOR, or SHA-256 edges.
Reqwest, CBOR, and cryptographic packages can remain transitive through
`ic-agent`.

The Registry-backed CloudEngine list builder requires both focused authority
features (or the convenience `host` feature):

```toml
[dependencies]
ic-query = { version = "0.62", default-features = false, features = ["cloud-engine-host", "subnet-catalog-host"] }
```

Enable `dashboard-host` when an embedder needs only the official Dashboard
REST reports, node-provider reward collection, CloudEngine provider and Type4
node collection, and the shared observed default-scope node-status cache:

```toml
[dependencies]
ic-query = { version = "0.62", default-features = false, features = ["dashboard-host"] }
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
ic-query = { version = "0.62", default-features = false, features = ["ic-state-host"] }
```

This exposes `LiveIcStateSource`, its focused source trait, and live/custom
builders. It enables `ic-agent`, Tokio, and URL validation without the cache
filesystem, Dashboard Reqwest transport, Registry Prost decoding, Futures,
direct CBOR decoding, or direct SHA-256 hashing.

Enable `cmc-host` when an embedder needs only authenticated Cycle Minting
Canister ICP/XDR and cycles reports:

```toml
[dependencies]
ic-query = { version = "0.62", default-features = false, features = ["cmc-host"] }
```

This enables `ic-agent` and direct CBOR certificate/witness decoding without
the cache filesystem or Registry protobuf graph. It has no direct ic-query
Futures, Reqwest, Prost, or SHA-256 edge; Reqwest and cryptographic packages
remain transitive through `ic-agent`.

Enable `icrc-host` for native ICRC ledger/index queries, certified-tip
verification, and the complete account-history cache:

```toml
[dependencies]
ic-query = { version = "0.62", default-features = false, features = ["icrc-host"] }
```

This leaves Dashboard, Registry, NNS, and SNS host adapters disabled and does
not enable ic-query's direct Reqwest or Prost dependencies. `ic-agent` retains
Reqwest transitively, so the package may still appear in the complete graph.

Enable `sns-host` for native SNS discovery, targeted reports, complete
proposal/neuron caches, reward checkpoints, and local checkpoint diffs:

```toml
[dependencies]
ic-query = { version = "0.62", default-features = false, features = ["sns-host"] }
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

Enable `certified-subnet-catalog-host` when an embedder needs certified Subnet
Catalog authority without the broader NNS Governance, proposal, neuron,
inventory, or derived-topology surface:

```toml
[dependencies]
ic-query = { version = "0.62", default-features = false, features = ["certified-subnet-catalog-host"] }
```

This feature includes `subnet-catalog-host` and adds certified Registry delta
collection, authenticated archive/replay, archive-bound catalog projection,
and the bounded certified-catalog cache. It directly enables CBOR certificate
decoding but not ic-query's Dashboard Reqwest edge. Live archive collection
remains explicit and bounded; local archive loading, projection, and certified
cache reads do not hide network calls.

Enable `nns-topology-host` when an embedder also needs the exact-version joined
NNS Subnet/node/operator/provider topology cache and source API:

```toml
[dependencies]
ic-query = { version = "0.62", default-features = false, features = ["nns-topology-host"] }
```

This feature includes `subnet-catalog-host` but not ic-query's direct optional
Dashboard Reqwest or CBOR certification edges. It does not expose the broader
component-cache topology summary builders; those require `nns-host` or the
complete `host` feature.

Enable `nns-host` for the complete NNS governance, proposal, neuron, Registry
inventory, component-cache, and derived topology host API:

```toml
[dependencies]
ic-query = { version = "0.62", default-features = false, features = ["nns-host"] }
```

This is a strict superset of both `nns-topology-host` and
`certified-subnet-catalog-host`. It directly includes the Registry Prost,
SHA-256, and CBOR dependencies needed by exact-version topology, endpoint
agreement, and certified Registry evidence. It has no direct ic-query
Dashboard Reqwest edge; Reqwest remains transitive through `ic-agent`.

The `certified-subnet-catalog-host` and `nns-host` library surfaces expose a
caller-runtime async
`fetch_nns_certified_registry_delta_batch_async` operation and pure
`validate_nns_certified_registry_delta_batch` validator. One call returns at
most one contiguous certified batch and reports `more_available`; it never
loops over later batches, writes a cache, or promotes Subnet Catalog
assurance. Chunk-referenced values in that batch are completed with
hash-verified `get_chunk` calls under fixed per-call, per-value, and aggregate
ceilings, with digest reuse and exact call/byte accounting preserved in report
schema 1. Each unique chunk is retained once in canonical digest order, and
the pure validator re-hashes its content, requires exact reference coverage,
and reconstructs every chunked value from the retained evidence. It checks
structural evidence returned by trusted custom sources; only the built-in live
transport cryptographically authenticates the raw certificate and witness.
Historical non-delete
mutations whose empty legacy protobuf value has no encoded content arm are
preserved as empty inline values, matching the official Registry transport.
Committed same-key mutations retain their stable order, and any retained
delete value remains raw evidence while replay ignores it when removing the
key.

Retained schema-1 delta reports can be checked again with
`reauthenticate_nns_certified_registry_delta_batch`. This local-only operation
verifies the raw mainnet certificate and mixed-tree commitment, decodes the
committed delta, and compares its complete certified contents with the report
before returning a sealed borrowed `NnsAuthenticatedRegistryDeltaBatch`.
Endpoint text remains explicit validated provenance but is not contacted.
Reauthentication does not make an historical snapshot currently fresh and
does not by itself restore a replay session or establish catalog assurance.

Multiple reauthenticated batches can be admitted through
`NnsAuthenticatedRegistryReplayBuilder`. The builder begins at Registry
version zero, accepts only sealed `NnsAuthenticatedRegistryDeltaBatch` values,
applies the existing explicit cumulative and state limits, and exposes
read-only replay progress. A complete exact-target sequence can be consumed
into the same `NnsAuthenticatedRegistryReplaySession` returned by live
bootstrap. Missing, out-of-order, incomplete, or oversized retained sequences
fail without acquiring that session capability. This path performs no network
or filesystem IO.

`NnsCertifiedRegistryArchiveManifestBuilder` adds the versioned, library-only
archive index contract. Schema 1 groups sealed batches into explicit completed-
target segments: the first is the version-zero bootstrap, while a batch after
completion opens an authenticated extension target. An empty segment may retain
a fresh certificate proving that the Registry version is unchanged without
changing the state digest. The builder hashes each report's canonical compact
JSON without buffering another encoded copy, applies explicit per-report and
total archive-byte ceilings before replay publication, and emits a manifest
only after the current segment completes. The manifest records strict batch and
segment order, targets, content digests, schema versions, accounting totals,
root-key identity, certificate-time bounds, replay commitments, and canonical
source endpoints.
A loaded manifest is never authority by itself: every retained report must be
size-checked, reauthenticated, replayed in order, and compared with a recomputed
manifest. Only the resulting sealed archive capability can enter the certified
catalog promotion path.

`NnsCertifiedRegistryArchivePublisher` and
`load_nns_certified_registry_archive` provide the explicit filesystem layer.
The publisher streams each canonical report once into a content-addressed,
owner-only managed object, synchronizes it, and publishes canonical
`manifest.json` only after exact-target replay completes. Paths are confined
beneath a caller-selected cache root; traversal, symbolic links, nonregular
files, and unsafe modes fail closed. Loading bounds the manifest and each
report before parsing, checks exact sizes and SHA-256 digests, processes one
report at a time, locally reauthenticates every certificate/witness/chunk set,
and accepts the archive only when a freshly recomputed manifest matches every
serialized field. Failed final publication preserves an existing complete
manifest. These low-level operations select no default archive path,
collection, refresh policy, lock, or CLI surface.

`NnsCertifiedRegistryArchivePublisher::resume` locally reloads and
reauthenticates an existing schema-1 archive under new caller-selected
cumulative limits, then accepts sealed extension batches without rewriting its
historical objects. It performs no source call and acquires no lock itself; a
caller using it directly must hold the dedicated archive lock across resume,
collection, and `finish`. Manifests with another schema identifier are rejected
without migration or a fallback reader and require an explicit new force
bootstrap.

`refresh_nns_certified_registry_archive_async` is the explicit live incremental
coordinator for an existing schema-1 archive. Its
`NnsCertifiedRegistryArchiveRefreshRequest` requires the collection, cumulative
replay/storage and confined-path policies. It rejects
non-mainnet and missing archives before source work, holds the archive lock
across local reauthentication and every bounded call, and atomically publishes
one complete successor segment. An unchanged Registry version is retained as
fresh authenticated evidence. Failure preserves the prior manifest; refresh is
never triggered by an archive load or ordinary catalog read-through.

`cleanup_nns_certified_registry_archive` is the separate, explicit local
maintenance boundary for objects left unreferenced by an interrupted
publication. It holds the same archive lock, fully reauthenticates the retained
archive, scans only regular files directly inside its exact `objects/`
directory, and applies caller-selected scan/count/byte ceilings before deleting
anything. It removes only files absent from the authenticated manifest,
synchronizes each successful deletion, and reports exact partial progress if a
later filesystem operation fails. Cleanup is never automatic and makes no
network call.

`bootstrap_nns_certified_registry_archive_async` is the explicit live
publication coordinator. Its request requires the source/time/replay policy,
caller-selected confined roots and archive storage ceilings. It rejects non-mainnet before filesystem or source work, holds one
dedicated archive refresh lock, reserves a worst-case batch before every call,
locally reauthenticates every returned report, and publishes the manifest only
after exact-target completion. The custom-source counterpart applies the same
mainnet reauthentication boundary. This is always a force bootstrap from
version zero; it is never invoked by archive or catalog loads and adds no
cleanup, default path, or CLI surface.

`nns-host` also exposes `NnsRegistryReplayState` and
`apply_nns_certified_registry_delta_batch` for pure, one-batch-at-a-time
reconstruction from version zero. Every apply requires caller-selected live
entry and raw content-byte ceilings and publishes atomically. It performs no
network or filesystem IO, does not choose a history budget, and is not
authority evidence without the validated certified reports that produced it.
`NnsRegistryReplaySession` adds cumulative version, batch, reported-call, and
response-byte ceilings and pins the first batch's certified latest version as
the exact target. If later reports observe a newer Registry version, the
session applies only through its original target. These pure admission limits
do not initiate or pre-budget source work. Sessions retain distinct source
endpoint strings in canonical order, certificate-time bounds, and a
deterministic digest chain over every admitted validated report. A separate
canonical state digest is exposed only after exact-target completion. One
public schema constant versions both commitments. These commitments do not
reauthenticate a custom source or establish catalog assurance by themselves.

A completed replay session can be projected in memory into canonical Subnet
Catalog rows. This pure diagnostic projection reads the replayed Subnet list,
routing authority, and referenced Subnet records at the pinned version and
reuses the live catalog's classification and routing-validation path. It uses
the complete present `canister_ranges_*` family when nonempty and explicitly
permits the legacy `routing_table` only to inspect a pre-shard historical
state. It does not by itself produce validated certified authority.

`project_nns_certified_subnet_catalog` is the authority boundary. It accepts
only a fully reauthenticated `NnsAuthenticatedRegistryArchive`, rechecks its
manifest against the sealed replay session, projects the exact reconstructed
state, and returns `NnsCertifiedSubnetCatalogAuthority`. Its required
`NnsCertifiedSubnetCatalogProjectionRequest` carries caller-owned validation
context, a maximum certificate age, and an explicit choice between an
authenticated historical target, which alone permits pre-shard legacy routing,
and requiring the selected target to be the newest version observed by every
archive batch with nonempty modern routing shards; there are no default
policies. Stale or knowingly
superseded archives fail before catalog record projection when prohibited. The
result keeps its private-field `ValidatedSubnetCatalog` attached to the archive
that proves it and exposes the exact age and version decision through
`.freshness()`.
Schema-1 provenance records archive/replay/report schema identities, root-key,
evidence-chain and complete-state digests, certificate-time bounds, and source
endpoints. Serializing those fields does not preserve authority: ordinary
`ValidatedSubnetCatalog::try_from_raw` validation always rejects `Certified`.
The projection performs no network or cache operation.

Certified projections use the same explicit-policy shape as ordinary catalog
loads. `load_nns_certified_subnet_catalog` accepts a caller-selected confined
root, dedicated cache directory, maximum envelope bytes, and one
`NnsCertifiedSubnetCatalogReadPolicy`: cache-only, publish-missing,
publish-missing-or-invalid, or force-publication. Publication holds persistent descriptor-based exclusion; there is no default
path, size, or publication policy.

Every policy is local-only. The loader bounds and strictly decodes an existing
envelope, freshly projects the supplied `NnsAuthenticatedRegistryArchive`, and
returns `NnsCertifiedSubnetCatalogLoadOutcome` only after an exact match.
Publication qualifies that same archive and current-use policy before acquiring
the cache's own lock and atomically replacing one canonical schema-1 envelope.
Missing-or-invalid recovery is limited to bounded malformed, noncanonical,
unsupported-schema, or archive-mismatched content; it never reclassifies a
filesystem, projection, serialization, or accounting failure. No policy
refreshes archive evidence or makes a network call.

Every successful outcome exposes its path and `cache_hit`,
`published_missing`, `published_invalid`, or `forced_publication` disposition.
`snapshot_authority()` returns the same stable Registry, catalog, endpoint, and
assurance identity as an ordinary validated catalog, excluding cache action.
`cache_evidence()` separately returns combined archive, certificate, and cache-
action diagnostics without duplicating the catalog snapshot. The serialized
`Certified` label and diagnostic DTO remain descriptive rather than authority
constructors: reloading authority still requires the matching authenticated
archive and fresh projection. The
ordinary Subnet Catalog remains schema 1, but its pre-1.0 shape is hard-cut to
require routing-source and per-record provenance fields. Older cache content is
invalid under the current reader and is refreshed only when the caller's
explicit cache policy authorizes invalid-content repair.

`bootstrap_nns_certified_registry_async` is the explicit live counterpart. It
starts at version zero on the caller's async runtime and reserves worst-case
capacity before each source call: one certified query, up to 64 chunk queries,
and up to 40 MiB of encoded responses. It has no default limits and returns a
complete `NnsAuthenticatedRegistryReplaySession` at the exact target. Only the
built-in mainnet-root-key verifier can construct this sealed wrapper. Use
`.replay_session()` for borrowed inspection or `.into_replay_session()` to
explicitly discard the capability. Custom-source bootstrap continues to return
the ordinary replay type because ic-query cannot reauthenticate its assertions.
The live session alone cannot be promoted to certified catalog authority. Its
evidence must first be retained through the authenticated archive boundary so
future use can reauthenticate the complete certificate/witness/chunk sequence.

`probe_nns_certified_registry_async` uses the same reservation and validation
loop for bounded sizing diagnostics. It returns either `Complete` or typed
`CapacityReached` status with the accumulated session and never makes the call
that would exceed capacity. A partial probe session is explicitly incomplete:
it is not a successful bootstrap, cache input, or catalog authority.

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
