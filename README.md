# ic-query

[![CI](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml/badge.svg)](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml)
[![docs.rs](https://docs.rs/ic-query/badge.svg)](https://docs.rs/ic-query)
[![License](https://img.shields.io/crates/l/ic-query.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.91.0-blue.svg)](Cargo.toml)

`ic-query` is a read-only Internet Computer reporting library.
`ic-query-cli` provides its `icq` command-line interface.

The project turns Registry, NNS, SNS, system-canister, ledger/index,
certificate, and official IC Dashboard responses into typed reports with
explicit provenance. It keeps live calls, cache reads, refreshes, and
local-only inspection visibly distinct.

## Supported reporting

| Family | Current surface |
| --- | --- |
| Official IC Dashboard | Bounded canister count/search pages, deployed canister metadata and upgrade history, bounded network metric time series and daily activity, boundary-node data-center aggregates, one-request observed node status with cached node/Subnet/provider views and typed provider assignment comparisons, and one-ledger ICRC total-supply/token-value history plus indexed account, holder, and transaction counts |
| NNS Registry | Registry version, Subnets, nodes, node operators, node providers, data centers, component topology diagnostics, and an exact-version joined topology library API |
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

# NNS Registry and cached topology diagnostics
icq nns registry version
icq nns topology refresh
icq nns topology summary

# Short-lived observed Dashboard status views
icq nns node status
icq nns subnet status --all
icq nns node-provider status --json

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
| NNS Registry | Exact-version joined Registry query evidence with explicit assurance | Current single-endpoint catalog collection is `uncertified_query`; shared Registry version prevents internal skew but does not certify an ordinary query response |
| NNS/SNS canisters | Read-only canister query responses | Paginated or sequential calls may span state changes |
| ICRC ledger/index | Ledger queries, index analytics, and archive callbacks | Index histories expose API exhaustion, not a stable snapshot version |
| ICRC tip certificate | Certificate and hash-tree evidence verified by the host adapter | Verification applies only when the ledger returns the required evidence |
| Cycle Minting Canister | Application-level certificate and hash-tree witness verified against the CMC and returned rate | Cycles per ICP is derived from the certified rate and the documented one-trillion-cycles-per-XDR protocol constant |
| Official IC Dashboard, including observed node status and ICRC analytics | Timestamped off-chain REST analytics | `certified: false`, `point_in_time_guaranteed: false`; default node scope excludes cloud-engine nodes, and an accepted ledger principal does not prove indexing coverage |

JSON reports keep raw identifiers, numeric fields, classifications, timestamps,
and explicit provenance. Text output may shorten or format values for people.
Report and persisted schemas are versioned independently and currently use
version `1`; before 1.0, incompatible shapes replace that version-1 contract
in place as hard cuts rather than adding compatibility branches or migrations.

See [IC Reporting Adapters](https://github.com/dragginzgame/ic-query/blob/main/docs/design/ic-reporting-adapters.md) for the
authority model and follow-up query rules.

## Command families

```text
icq cache status

icq ic canister count|info|page
icq ic metrics <metric>
icq ic network boundary-node-data-centers
icq ic network daily-stats

icq nns data-center info|list|refresh
icq nns governance economics|maturity-modulation|metrics|reward-event
icq nns neuron cache|info|list|refresh
icq nns node info|list|refresh|status
icq nns node-operator info|list|refresh
icq nns node-provider info|list|refresh|status
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

The top-level `--network` option supplies network identity to NNS, SNS, and
system-canister commands. Built-in sources and caches currently accept only
the mainnet `ic` identity.

Dashboard canister and ICRC commands identify their target using a stable
entity id and an explicit API endpoint; Dashboard metric and network-resource
commands use an official resource identity and endpoint. These families reject
the global `--network` option; use the command’s `--source-endpoint` option when
an endpoint override is needed. Every live endpoint must be a credential-free
HTTP(S) base URL with a host and no query or fragment. Official Dashboard
requests do not follow redirects, so provenance always names the endpoint that
returned the response.

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

Dashboard count, page, metric, daily-statistics, boundary-node data-center, and
ICRC analytics commands always make exactly one REST request. The shared live
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
days and 1,000 rows. None of these commands creates a cache.

Observed node status is the bounded exception: one unfiltered Dashboard
`/nodes` request creates a complete network-level snapshot capped at 10,000
rows and 8 MiB. Node, Subnet, and node-provider status commands project that
same identity, reuse it for 60 seconds, and visibly refresh missing, invalid,
or stale content. View targets and `--all` never create separate caches;
`--refresh` forces replacement. The snapshot preserves raw status/type fields,
states that it is not certified or point-in-time, and records that the
Dashboard default public-mainnet scope excludes cloud-engine nodes.

Native Registry, NNS, SNS, ICRC, and CMC calls also cap every `ic-agent`
response body at 8 MiB. This is a per-call transport bound; paged collection,
atomic cache publication, and explicit refresh policies retain their existing
report-specific row and call limits.

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
ic-query = { version = "0.29", default-features = false }
```

Native tools that need live calls, filesystem caches, refreshes, or custom
source adapters enable `host`:

```toml
[dependencies]
ic-query = { version = "0.29", default-features = false, features = ["host"] }
```

The no-default build is checked for `wasm32-unknown-unknown` without Clap,
`ic-agent`, Reqwest, Tokio, or `futures`. This is a host-dependency boundary,
not a `no_std` promise.

Public report families are exposed from:

- `ic_query::cache` for shared models, with inventory builders and rendering
  under the `host` feature
- `ic_query::ic`
- `ic_query::icrc`
- `ic_query::nns`
- `ic_query::sns`
- `ic_query::subnet_catalog`
- `ic_query::system::cmc`

Built-in host calls use one adapter per authority family:
`LiveIcSource`, `LiveIcrcSource`, `LiveNnsSource`, `LiveSnsSource`, and
`LiveCmcSource`.
Report-specific capability traits let fixtures, mirrors, proxies, and
pre-collected sources reuse the same validation and projection path.

Library builders do not write to stdout or stderr. Paged refresh APIs can emit
typed `QueryProgressEvent` values to a caller-provided sink; terminal rendering
remains an `ic-query-cli` responsibility.

Enable the narrower `subnet-catalog-host` feature when a native embedder needs
only live/cache Subnet catalog behavior. It keeps the IC agent, Registry
decoding, runtime bridge, capability-filesystem dependencies, and other cache
dependencies required by that API while leaving `ic-query`'s direct optional
Dashboard `reqwest` transport and `serde_cbor` certification dependencies
disabled. Because the feature still includes `ic-agent`, both packages may
remain in its transitive dependency graph. The full `host` feature remains the
choice for all reporting adapters and is a strict superset.

The Subnet Catalog API separates serde-facing `RawSubnetCatalog` data from
private-field `ValidatedSubnetCatalog` evidence. Explicit load policies return
both the validated catalog and an observable cache disposition; validated
canister resolution returns the matched range, Registry version, catalog
digest, and full provenance together. Single-endpoint live collection is
always labelled `CatalogAssurance::UncertifiedQuery`. Async embedders can call
`fetch_subnet_catalog_async` on their own runtime; synchronous builders retain
the runtime adapter and may use a scoped helper thread when invoked inside an
existing Tokio runtime.

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
