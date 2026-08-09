# ic-query

[![CI](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml/badge.svg)](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml)
[![docs.rs](https://docs.rs/ic-query/badge.svg)](https://docs.rs/ic-query)
[![License](https://img.shields.io/crates/l/ic-query.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.91.0-blue.svg)](Cargo.toml)

`ic-query` is a read-only Internet Computer reporting library.
`ic-query-cli` provides its `icq` command-line interface.

The project turns CloudEngine, Registry, NNS, SNS, system-canister,
ledger/index, certificate, and official IC Dashboard responses into typed
reports with explicit provenance. It keeps live calls, cache reads, refreshes,
and local-only inspection visibly distinct.

## Supported reporting

| Family | Current surface |
| --- | --- |
| Certified IC state | Complete authenticated API boundary-node identities, domains, and IPv4/IPv6 configuration from one certified state tree |
| Official IC Dashboard | Bounded canister count/search pages, deployed canister metadata and upgrade history, bounded network metric time series and daily activity, boundary-node data-center aggregates, exact/one-page replica releases, exact/one-page/aggregate node-provider rewards, one-request observed default-scope and explicit Type4 node status, cached default-scope node/Subnet/provider views with typed provider assignment comparisons, and one-ledger ICRC total-supply/token-value history plus indexed account, holder, and transaction counts |
| CloudEngine | Registry-backed CloudEngine Subnet inventory with bounded public operator bindings, exact one-Subnet operator details, public network fee and bounded marketplace prices, one-request official Dashboard provider footprint and exact provider detail, plus explicit Type4 node health, assignment, and exact detail |
| NNS Registry | Certified latest version, bounded exact-target replay and retained archives, archive-bound certified Subnet Catalog authority, Subnets, nodes, node operators, node providers, data centers, component topology diagnostics, and an exact-version joined topology library API |
| NNS Governance | Proposals, publicly readable neurons, economics, metrics, latest reward event, and maturity modulation |
| SNS | Cached joined discovery, targeted metadata, token and nervous-system parameters, bounded Governance metrics, swap and upgrade state, Root canister inventory and health, proposals, fixed-size neuron collections, exact permission/followee neuron detail, bracketed API-exhausted maturity checkpoints, and local reward-event reconciliation |
| ICRC | Capabilities, token metadata, balances, allowances, index discovery, ledger and account transactions, archives, block types, tip certificates, and bounded official total-supply, external token-value, and indexed-count analytics |
| System canisters | Certified Cycle Minting Canister ICP/XDR rates and exact cycles-per-ICP derivation |

The living [Roadmap to 1.0](https://github.com/dragginzgame/ic-query/blob/main/docs/roadmap/1.0.md) records the broader reporting
surface, current coverage estimates, and the remaining work.

## Install

From this checkout:

```bash
make install
```

The install target replaces an existing `icq` binary, so repeated development
installs do not need a separate Cargo `--force` option.

From crates.io:

```bash
cargo install ic-query-cli
```

Local `make patch`, `make minor`, and `make major` runs execute the complete
release gate and remove this workspace's Cargo build artifacts only after CI
passes and version metadata is updated. A failed gate retains `target/` for
diagnosis. Cleanup failure after a successful bump is reported as a warning
rather than disguising the release result. CI helper scripts likewise remove
only their own exact temporary paths. They do not sweep shared `/tmp` or remove
the shared Cargo download cache.

## Quick start

```bash
# Official Dashboard canister metadata
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai
icq ic canister count --has-name true
icq ic canister page --query ledger --limit 25

# Official Dashboard network metrics
icq ic metrics instruction-rate
icq ic metrics ic-node-count --json

# Official Dashboard network resources
icq ic network boundary-node-data-centers
icq ic network daily-stats

# Certified API boundary-node configuration
icq ic api-boundary-node list
icq ic api-boundary-node list --json

# Official Dashboard replica release records
icq ic replica-version list --limit 25
icq ic replica-version info e3d101b22ae3fa02aca737f9fb96cc6c4ca83ac3

# Public CloudEngine reports
icq cloud-engine list
icq cloud-engine info 2nl67-oqoc5-cmocj-otlhq-kr2kr-53hov-drrds-7ihcs-fhomv-2eyvu-6qe
icq cloud-engine prices --json
icq cloud-engine node list
icq cloud-engine node info 53amq-7hjxu-6lxaj-o2sp6-kmngy-qa22h-b7bo6-oeyyn-fkqnv-7tauf-7qe
icq cloud-engine provider list
icq cloud-engine provider info rbn2y-6vfsb-gv35j-4cyvy-pzbdu-e5aum-jzjg6-5b4n5-vuguf-ycubq-zae

# NNS Registry and cached topology diagnostics
icq nns registry version
icq nns topology refresh
icq nns topology summary

# Short-lived observed Dashboard status views
icq nns node status
icq nns subnet status --all
icq nns node-provider status --json

# Official Dashboard node-provider reward records
icq nns node-provider reward info 7562
icq nns node-provider reward list --limit 25
icq nns node-provider reward history --json

# Governance reports
icq nns proposal list --limit 25
icq nns neuron list --limit 25
icq nns governance economics

# Deployed SNS reports
icq sns list
icq sns list --all
icq sns refresh
icq sns canister list 1
icq sns metrics 1
icq sns upgrade 1
icq sns proposal list 1 --limit 25
icq sns neuron info 1 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
icq sns reward checkpoint 1 --json
icq sns reward diff before-checkpoint.json after-checkpoint.json --json

# Generic ICRC reports
icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc account balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa
icq icrc analytics account count mxzaz-hqaaa-aaaar-qaada-cai
icq icrc analytics holder count mxzaz-hqaaa-aaaar-qaada-cai
icq icrc analytics token-values mxzaz-hqaaa-aaaar-qaada-cai
icq icrc analytics total-supply mxzaz-hqaaa-aaaar-qaada-cai
icq icrc analytics transaction count mxzaz-hqaaa-aaaar-qaada-cai

# Native system-canister reports
icq system xdr
icq system cycles --json

# Local cache inventory
icq cache status
```

Text is the default human-facing format. Use `--json` on report commands
for raw, script-friendly fields:

```bash
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --json
icq nns topology summary --json
```

Run `icq help`, `icq help <path>`, or append `--help` to a command for its
current options and collection mode. A command namespace without its next
operation displays the same complete local help as its explicit `help`
subcommand, such as `icq sns reward`. Every
`Commands` section is ordered alphabetically. The complete command map and
cache behavior are documented in
[CLI Usage](https://github.com/dragginzgame/ic-query/blob/main/docs/cli-usage.md).

## Authority and freshness

An “official” source is not automatically certified or point-in-time
consistent. Reports preserve the authority and guarantees the source can
actually make:

| Source | Evidence represented | Important limit |
| --- | --- | --- |
| Certified IC state tree | API boundary-node principals, domains, addresses, and certificate time authenticated against the built-in mainnet root key | Configuration does not prove operational health, reachability, HTTP-gateway membership, ownership, or physical location |
| NNS Registry | Authenticated certified latest-version and bounded contiguous delta-batch evidence, archive-bound exact-state catalog authority, plus exact-version joined Registry query evidence with explicit assurance | A single batch does not reconstruct Registry state; only a complete reauthenticated archive can promote a certified catalog, ordinary `get_value` reads remain uncertified, and endpoint agreement is not cryptographic certification |
| NNS/SNS canisters | Read-only canister query responses | Paginated or sequential calls may span state changes |
| ICRC ledger/index | Ledger queries, index analytics, and archive callbacks | Index histories expose API exhaustion, not a stable snapshot version |
| ICRC tip certificate | Certificate and hash-tree evidence verified by the host adapter | Verification applies only when the ledger returns the required evidence |
| Cycle Minting Canister | Application-level certificate and hash-tree witness verified against the CMC and returned rate | Cycles per ICP is derived from the certified rate and the documented one-trillion-cycles-per-XDR protocol constant |
| CloudEngine control plane | Ordinary public canister query responses from the fixed control-plane and resolved operator canister; list inventory separately comes from the versioned Registry Subnet Catalog | `certified: false`, `point_in_time_guaranteed: false` for control-plane observations; per-row calls do not become part of the Registry snapshot |
| Official IC Dashboard, including CloudEngine provider footprint and explicit Type4 nodes, observed default-scope node status, replica releases, node-provider rewards, and ICRC analytics | Timestamped off-chain REST analytics | `certified: false`, `point_in_time_guaranteed: false`; provider aggregates and Type4 node observations are separately timed, default node scope still excludes CloudEngine nodes, release records do not prove the binary currently running, reward offset pages may overlap, and an accepted ledger principal does not prove indexing coverage |

JSON reports keep raw identifiers, numeric fields, classifications, timestamps,
and explicit provenance. Text output may shorten or format values for people.
Report and persisted schemas are versioned independently. Every current schema
identifier remains `1` before 1.0; incompatible shapes replace that contract
in place without compatibility readers or automatic migrations.

See [IC Reporting Adapters](https://github.com/dragginzgame/ic-query/blob/main/docs/design/ic-reporting-adapters.md) for the
authority model and follow-up query rules.

## Command families

```text
icq cache status

icq cloud-engine info <subnet-id>
icq cloud-engine list
icq cloud-engine node info|list
icq cloud-engine prices
icq cloud-engine provider info|list

icq ic api-boundary-node list
icq ic canister count|info|page
icq ic metrics <metric>
icq ic network boundary-node-data-centers
icq ic network daily-stats
icq ic replica-version info|list

icq nns data-center info|list|refresh
icq nns governance economics|maturity-modulation|metrics|reward-event
icq nns neuron cache|info|list|refresh
icq nns node info|list|refresh|status
icq nns node-operator info|list|refresh
icq nns node-provider info|list|refresh|status
icq nns node-provider reward history|info|list
icq nns proposal cache|info|list|refresh
icq nns registry version
icq nns subnet info|list|refresh|status
icq nns topology capacity|check|coverage|gaps|providers|refresh|regions|summary|versions

icq sns list|refresh
icq sns info|metrics|parameters|swap|token|upgrade <SNS>
icq sns canister list <SNS>
icq sns neuron cache list
icq sns neuron cache status <SNS>
icq sns neuron info <SNS> <neuron-id>
icq sns neuron list|refresh <SNS>
icq sns proposal cache list
icq sns proposal cache status <SNS>
icq sns proposal info <SNS> <proposal-id>
icq sns proposal list|refresh <SNS>
icq sns reward checkpoint <SNS>
icq sns reward diff <before.json> <after.json>

icq icrc account allowance|balance
icq icrc account transaction cache|list|page|refresh
icq icrc analytics account count <ledger-canister-id>
icq icrc analytics holder count <ledger-canister-id>
icq icrc analytics token-values <ledger-canister-id>
icq icrc analytics total-supply <ledger-canister-id>
icq icrc analytics transaction count <ledger-canister-id>
icq icrc ledger archives|block-types|capabilities|index|tip-certificate|token|transactions

icq system cycles|xdr
```

The top-level `--network` option supplies network identity to CloudEngine, NNS,
SNS, and system-canister commands. Built-in sources and caches currently
accept only the mainnet `ic` identity.

Dashboard canister and ICRC commands identify their target using a stable
entity id and an explicit API endpoint; Dashboard metric and network-resource
commands use an official resource identity and endpoint. These families reject
the global `--network` option; use the command’s `--source-endpoint` option when
an endpoint override is needed. Every live endpoint must be a credential-free
HTTP(S) base URL with a host and no query or fragment. Official Dashboard
requests do not follow redirects, so provenance always names the endpoint that
returned the response.
`cloud-engine node` and `cloud-engine provider` remain below the top-level
CloudEngine family and therefore honor the global network identity, currently
only `ic`; their local `--source-endpoint` still identifies the official
Dashboard authority.

The certified API boundary-node command is fixed to mainnet and rejects the
global `--network` option. Its `--source-endpoint` selects the mainnet IC API
endpoint used for `read_state`; the report records that endpoint and the fixed
Registry effective canister id used only to route the request.

## Collection and cache behavior

Every data-producing command follows one documented collection mode:

| Mode | Network access | Cache writes |
| --- | --- | --- |
| Live query | Always | Never |
| Cache-backed, refresh if missing | When the complete cache is absent or its local content is recoverably invalid | Publishes a validated complete snapshot |
| Cache-backed, refresh if stale | When the complete cache is absent, recoverably invalid, or older than its documented policy | Publishes a validated complete snapshot |
| Cache-preferred, live fallback | Only when cached data cannot satisfy the lookup | Only an explicit refresh writes complete collections |
| Local-only inspection | Never | Never |
| Forced refresh | Always | Atomically replaces the prior complete snapshot after validation |

Dashboard count, page, metric, daily-statistics, boundary-node data-center,
replica-version, CloudEngine node/provider, and ICRC analytics commands always
make exactly one REST request. The shared live
transport rejects successful response bodies larger than 8 MiB, checking both
declared and streamed sizes before JSON decoding. Indexed counts request no
account, holder, or transaction rows. A page returns at most 100 canister
summaries and never follows its cursors automatically. A metric query
defaults to one hour at a five-minute step and is capped at 1,000 observations
per series. Daily statistics default to seven days and are capped at one year
and 366 rows. The boundary-node report consumes one non-paginated data-center
resource and makes no per-location calls. Total-supply analytics default to 30
days at a daily step, retain raw ledger base units, and are capped at 1,000
observations. Token-value analytics default to 24 hours and are capped at 90
days and 1,000 rows. Replica-version pages default to 50 rows, cap at 100, and
follow an offset only when supplied explicitly; exact info preserves the raw
release summary. These release-election records are not proof of the replica
binary currently running on a Subnet. None of these commands creates a cache.

Observed node status is the bounded exception: one unfiltered Dashboard
`/nodes` request creates a complete network-level snapshot capped at 10,000
rows and 8 MiB. Node, Subnet, and node-provider status commands project that
same identity, reuse it for 60 seconds, and visibly refresh missing, invalid,
or stale content. View targets and `--all` never create separate caches;
`--refresh` forces replacement. The snapshot preserves raw status/type fields,
states that it is not certified or point-in-time, and records that the
Dashboard default public-mainnet scope excludes cloud-engine nodes.

Native CloudEngine, Registry, NNS, SNS, ICRC, and CMC calls also cap every `ic-agent`
response body at 8 MiB. This is a per-call transport bound; paged collection,
atomic cache publication, and explicit refresh policies retain their existing
report-specific row and call limits.

`ic api-boundary-node list` makes one response-bounded `read_state` request
for the complete `api_boundary_nodes` subtree. The agent authenticates one
certificate and the report preserves its raw Unix-nanosecond time plus a
Unix-second projection. All rows share that certified time; no cache or
per-node follow-up is used.

`cloud-engine info` and `cloud-engine prices` are live and uncached. `info`
makes one control-plane query when no operator is registered and exactly five
native queries when an operator exists; `prices` makes exactly two calls and
accepts at most 1,000 rows. `cloud-engine list` uses the complete Registry
Subnet Catalog cache policy and then attempts one exact public control-plane
binding lookup per Registry CloudEngine row, capped at 100. Registry and
control-plane provenance remain separate, including per-row failures. Claimed
domains are capped at 100.

`cloud-engine provider list` makes one Dashboard request for the complete
node-provider resource, validates at most 1,000 rows, and then retains rows
with explicit CloudEngine counts or locations. `provider info` makes one exact
request and preserves valid providers with no current CloudEngine evidence.
Both are live and uncached; their Dashboard provenance remains distinct from
Registry inventory and native control-plane observations.

`cloud-engine node list` makes one Dashboard request with the explicit
`Type4` reward filter and repeated filters for all four current operational
statuses. It accepts at most 10,000 rows, optionally applies one exact provider
filter, preserves null CloudEngine Subnet assignment, and reports status,
provider, assigned-Subnet, and unassigned counts. `node info` makes one exact
node-id request and rejects a returned non-Type4 row. Both are live and
uncached; they do not reuse the default-scope node-status cache or reconcile a
separately timed provider aggregate.

Successful SNS metrics queries default to a 30-day proposal-count window
capped at 365 days. They make three targeted client requests, preserve
Governance-cached treasury and voting-power timestamps, and do not scan
transactions, fan out, or create a cache. Paged proposal, neuron, and
account-history collections retain refresh attempt state. Failed or capped
refreshes do not replace the last complete snapshot.

Subnet Catalog callers select `CacheOnly`, refresh-missing,
refresh-missing-or-invalid, refresh-older-than, or force-refresh behavior and
receive the exact `CacheDisposition` used. Catalog loads validate fixed
mainnet/Registry identity, raw Registry Subnet kinds, classification and
resolver policy identity, timestamps, canonical ordering, and the canonical
payload digest before returning a `ValidatedSubnetCatalog`. The digest detects
an inconsistent payload; it is not a signature and does not promote the
current `UncertifiedQuery` assurance. Bounded NNS Registry inventory
read-through operations retain their owner-selected invalid-content repair.
Exact-version topology and ICRC account-history library callers receive the
same behavior only through explicitly selected read-through APIs. Direct cache
loads, filesystem failures, and complete Governance history caches remain
strict. Cache-status operations stay local: family-specific status reports
validate their owned snapshots, while the bounded top-level inventory inspects
generic headers only.

Library Subnet Catalog refresh policies use `CatalogSourceSelection`: one
endpoint keeps `UncertifiedQuery`, while an explicit two-to-three-endpoint
selection requires distinct hostnames and exact Registry-version/payload
agreement. Successful provenance records canonical endpoints, an agreement
digest when applicable, and exact Registry query-call counts. Agreement does
not become certified evidence and never falls back to one endpoint on a
mismatch.

`sns list` uses a one-hour joined catalog cache containing Governance metadata
and raw Swap lifecycle evidence, so consecutive fresh reads make no live calls.
Missing, stale, malformed, incompatible, or semantically invalid catalogs
produce visible refresh progress and are replaced only after a valid complete
snapshot is ready; failed refreshes leave the prior file untouched. Cache-only
and family-specific cache-status operations remain local and report invalid
evidence directly.
The default view includes lifecycle `3` (`committed`, successfully launched)
SNSes; `sns list --all` also shows failed/aborted, pending, unknown, and
lifecycle-query-error rows. `sns refresh` forces replacement. Targeted SNS
commands keep their bounded targeted discovery and never refresh the all-SNS
catalog.
The exact-version joined topology cache uses one refresh lock and atomic
replacement without a separate attempt sidecar. Collection limits and cursors
are operation controls; sorts, view limits, verbosity, and output format do
not change snapshot identity.

The CLI uses one user-level cache root in every working directory. It selects
the first non-empty source:

1. `ICQ_CACHE_ROOT`, which must be an absolute path;
2. `$XDG_CACHE_HOME/ic-query`; or
3. `$HOME/.cache/ic-query`.

It does not inspect project files or read and migrate former project-local
`.icq` directories. Cache semantics and recovery rules are defined in
[Cache Policy](https://github.com/dragginzgame/ic-query/blob/main/docs/design/cache-policy.md).
Network-scoped cache paths consistently begin with
`<cache-root>/<domain>/<network>/...`.

Managed cache loads, discovery, refresh locks, and publication are confined to
that root. On Unix, symbolic links, path escapes, nonregular managed files,
group/other-accessible directories, and files not using mode `0600` are
rejected. New managed directories use mode `0700`; new cache and lock files use
mode `0600`. These authority failures are not treated as invalid JSON that a
read-through call may silently replace. Explicit caller-selected output files
are outside this cache policy. The `0.29.1` hard cut does not migrate or loosen
older permissive caches: remove the old cache root or restrict its directories
and files before use.

Use `icq cache status` to inspect known complete caches across that root,
including generic header integrity, separate fresh/stale/unmanaged/unknown age,
sizes, stale thresholds, and automatic/explicit/missing-only invalid-content
recovery policy. The bounded inventory explicitly reports that it did not
perform family-specific semantic validation. It also reports active, stale,
and invalid refresh locks without live calls, process probes, full history
scans, or cache mutation.

## Library

Use `ic-query` for typed requests, reports, validation, cache behavior, source
adapters, and renderers without spawning `icq`.

Pure DTO and rendering use has no host dependencies:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false }
```

Native tools that need live calls, filesystem caches, refreshes, or custom
source adapters enable `host`:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["host"] }
```

The no-default build is checked for `wasm32-unknown-unknown` without Clap,
`ic-agent`, Reqwest, Tokio, or `futures`. This is a host-dependency boundary,
not a `no_std` promise.

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

Library builders do not write to stdout or stderr. Paged refresh APIs can emit
typed `QueryProgressEvent` values to a caller-provided sink; terminal rendering
remains an `ic-query-cli` responsibility.

Enable `cloud-engine-host` when an embedder needs only the public CloudEngine
operator and marketplace reports:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["cloud-engine-host"] }
```

This enables `ic-agent`, Tokio, and URL validation without ic-query's direct
cache-filesystem, Registry Prost, Futures, Reqwest, CBOR, or SHA-256 edges.
Reqwest, CBOR, and cryptographic packages can remain transitive through
`ic-agent`.

The Registry-backed CloudEngine list builder requires both focused authority
features (or the convenience `host` feature):

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["cloud-engine-host", "subnet-catalog-host"] }
```

Enable `dashboard-host` when an embedder needs only the official Dashboard
REST reports, node-provider reward collection, CloudEngine provider and Type4
node collection, and the shared observed default-scope node-status cache:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["dashboard-host"] }
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
ic-query = { version = "0.36", default-features = false, features = ["ic-state-host"] }
```

This exposes `LiveIcStateSource`, its focused source trait, and live/custom
builders. It enables `ic-agent`, Tokio, and URL validation without the cache
filesystem, Dashboard Reqwest transport, Registry Prost decoding, Futures,
direct CBOR decoding, or direct SHA-256 hashing.

Enable `cmc-host` when an embedder needs only authenticated Cycle Minting
Canister ICP/XDR and cycles reports:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["cmc-host"] }
```

This enables `ic-agent` and direct CBOR certificate/witness decoding without
the cache filesystem or Registry protobuf graph. It has no direct ic-query
Futures, Reqwest, Prost, or SHA-256 edge; Reqwest and cryptographic packages
remain transitive through `ic-agent`.

Enable `icrc-host` for native ICRC ledger/index queries, certified-tip
verification, and the complete account-history cache:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["icrc-host"] }
```

This leaves Dashboard, Registry, NNS, and SNS host adapters disabled and does
not enable ic-query's direct Reqwest or Prost dependencies. `ic-agent` retains
Reqwest transitively, so the package may still appear in the complete graph.

Enable `sns-host` for native SNS discovery, targeted reports, complete
proposal/neuron caches, reward checkpoints, and local checkpoint diffs:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["sns-host"] }
```

This leaves Dashboard, Registry, NNS, system-canister, and native ICRC host
adapters disabled. It shares the ICRC token-metadata mechanics required to
inspect SNS ledgers, but does not enable the native ICRC report or cache
surface. There is no direct ic-query edge to Reqwest, Prost, CBOR, or SHA-256;
`ic-agent` retains Reqwest, CBOR, and cryptographic packages transitively.

Enable the narrower `subnet-catalog-host` feature when a native embedder needs
only live/cache Subnet catalog behavior. It keeps the IC agent, Registry
decoding, runtime bridge, capability-filesystem dependencies, and other cache
dependencies required by that API while leaving `ic-query`'s direct optional
Dashboard `reqwest` transport and `serde_cbor` certification dependencies
disabled. Because the feature still includes `ic-agent`, both packages may
remain in its transitive dependency graph. The full `host` feature remains the
choice for all reporting adapters and is a strict superset.

Enable `certified-subnet-catalog-host` when an embedder needs certified Subnet
Catalog authority without the broader NNS Governance, proposal, neuron,
inventory, or derived-topology surface:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["certified-subnet-catalog-host"] }
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
ic-query = { version = "0.36", default-features = false, features = ["nns-topology-host"] }
```

This feature includes `subnet-catalog-host` but not ic-query's direct optional
Dashboard Reqwest or CBOR certification edges. It does not expose the broader
component-cache topology summary builders; those require `nns-host` or the
complete `host` feature.

Enable `nns-host` for the complete NNS governance, proposal, neuron, Registry
inventory, component-cache, and derived topology host API:

```toml
[dependencies]
ic-query = { version = "0.36", default-features = false, features = ["nns-host"] }
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
replay/storage, confined path, and lock-staleness policies. It rejects
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
caller-selected confined roots, archive storage ceilings, and lock-staleness
policy. It rejects non-mainnet before filesystem or source work, holds one
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
routing table, and referenced Subnet records at the pinned version and reuses
the live catalog's classification and routing-validation path. It does not by
itself produce validated certified authority.

`project_nns_certified_subnet_catalog` is the authority boundary. It accepts
only a fully reauthenticated `NnsAuthenticatedRegistryArchive`, rechecks its
manifest against the sealed replay session, projects the exact reconstructed
state, and returns `NnsCertifiedSubnetCatalogAuthority`. Its required
`NnsCertifiedSubnetCatalogProjectionRequest` carries caller-owned validation
context, a maximum certificate age, and an explicit choice between an
authenticated historical target and requiring the newest version observed by
every archive batch; there are no default policies. Stale or knowingly
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
publish-missing-or-invalid, or force-publication. Publication policies also
carry the explicit stale-lock age; there is no default path, size, or policy.

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
`authority_evidence()` returns a compact persistable Registry, catalog,
archive, certificate, endpoint, assurance, and cache-action identity without
duplicating the catalog snapshot. The serialized `Certified` label and evidence
DTO remain descriptive rather than authority constructors: reloading authority
still requires the matching authenticated archive and fresh projection. The
ordinary schema-1 Subnet Catalog cache is unchanged.

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
both the validated catalog and an observable cache disposition; validated
canister resolution returns the matched range, Registry version, catalog
digest, and full provenance together. Single-endpoint live collection is
always labelled `CatalogAssurance::UncertifiedQuery`. Async embedders can call
`fetch_subnet_catalog_async`, `load_subnet_catalog_async`, or
`refresh_subnet_catalog_async` on their own runtime. Dropping an async refresh
releases its owned lock without publishing. Synchronous adapters may use a
scoped helper thread when invoked inside an existing Tokio runtime.
Only schema-1 catalog files are accepted; an explicit read policy may refresh
unsupported ordinary cache content, but no migration or legacy reader is
retained.

See [Library Usage](https://github.com/dragginzgame/ic-query/blob/main/docs/library-usage.md) for complete examples and feature
guidance.

## Documentation

- [Documentation index](https://github.com/dragginzgame/ic-query/blob/main/docs/README.md)
- [CLI usage and collection modes](https://github.com/dragginzgame/ic-query/blob/main/docs/cli-usage.md)
- [Library usage](https://github.com/dragginzgame/ic-query/blob/main/docs/library-usage.md)
- [Roadmap to 1.0](https://github.com/dragginzgame/ic-query/blob/main/docs/roadmap/1.0.md)
- [0.22 structural consolidation](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.22/0.22-design.md)
- [0.23 bounded SNS completeness](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.23/0.23-design.md)
- [0.24 bounded SNS Governance metrics](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.24/0.24-design.md)
- [0.25 fuller fixed-size SNS neuron evidence](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.25/0.25-design.md)
- [0.26 SNS maturity reward evidence](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.26/0.26-design.md)
- [0.27 bounded official ICRC analytics](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.27/0.27-design.md)
- [0.28 observed IC node and Subnet status](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.28/0.28-design.md)
- [0.29 Subnet Catalog authority and embedder hardening](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.29/0.29-design.md)
- [0.30 certified Registry evidence](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.30/0.30-design.md)
- [0.31 public CloudEngine reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.31/0.31-design.md)
- [0.32 bounded replica-version reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.32/0.32-design.md)
- [0.33 certified API boundary-node reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.33/0.33-design.md)
- [0.34 CloudEngine provider reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.34/0.34-design.md)
- [0.35 CloudEngine Type4 node reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.35/0.35-design.md)
- [0.36 node-provider reward reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/0.36/0.36-design.md)
- [IC Dashboard canister reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/ic-dashboard-canister-reporting.md)
- [IC Dashboard network metrics](https://github.com/dragginzgame/ic-query/blob/main/docs/design/ic-dashboard-network-metrics.md)
- [IC Dashboard daily statistics](https://github.com/dragginzgame/ic-query/blob/main/docs/design/ic-dashboard-daily-stats.md)
- [IC Dashboard boundary-node reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/ic-dashboard-boundary-node-reporting.md)
- [Exact-version NNS Subnet topology](https://github.com/dragginzgame/ic-query/blob/main/docs/design/nns-subnet-topology.md)
- [SNS Root canister inventory and health](https://github.com/dragginzgame/ic-query/blob/main/docs/design/sns-root-canister-reporting.md)
- [Certified CMC system reporting](https://github.com/dragginzgame/ic-query/blob/main/docs/design/cmc-system-reporting.md)
- [Release ledger](https://github.com/dragginzgame/ic-query/blob/main/CHANGELOG.md)

## Scope

`ic-query` is read-only metadata and evidence tooling. It does not replace
`dfx`, expose arbitrary Candid invocation, or perform canister, governance,
ledger, or network mutations.

Deployment and orchestration projects can call the CLI or use the Rust library
when they need these reporting contracts. One downstream integration is
[Canic](https://github.com/dragginzgame/canic).
