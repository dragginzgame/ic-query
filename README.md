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
| Official IC Dashboard | Bounded canister count/search pages, deployed canister metadata and upgrade history, bounded network metric time series and daily activity, and boundary-node data-center aggregates |
| NNS Registry | Registry version, Subnets, nodes, node operators, node providers, data centers, component topology diagnostics, and an exact-version joined topology library API |
| NNS Governance | Proposals, publicly readable neurons, economics, metrics, latest reward event, and maturity modulation |
| SNS | Discovery, metadata, token and nervous-system parameters, bounded Governance metrics, swap and upgrade state, Root canister inventory and health, proposals, and fixed-size neuron state |
| ICRC | Capabilities, token metadata, balances, allowances, index discovery, ledger and account transactions, archives, block types, and tip certificates |
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

# Governance reports
icq nns proposal list --limit 25
icq nns neuron list --limit 25
icq nns governance economics

# Deployed SNS reports
icq sns list
icq sns canister list 1
icq sns metrics 1
icq sns upgrade 1
icq sns proposal list 1 --limit 25

# Generic ICRC reports
icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc account balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa

# Native system-canister reports
icq system xdr
icq system cycles --json
```

Text is the default human-facing format. Use `--json` on report commands
for raw, script-friendly fields:

```bash
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --json
icq nns topology summary --json
```

Run `icq help`, `icq help <path>`, or append `--help` to a command for its
current options and collection mode. The complete command map and cache behavior
are documented in [CLI Usage](https://github.com/dragginzgame/ic-query/blob/main/docs/cli-usage.md).

## Authority and freshness

An “official” source is not automatically certified or point-in-time
consistent. Reports preserve the authority and guarantees the source can
actually make:

| Source | Evidence represented | Important limit |
| --- | --- | --- |
| NNS Registry | Versioned on-chain Registry records | Joined topology is authoritative only for the recorded Registry version |
| NNS/SNS canisters | Read-only canister query responses | Paginated or sequential calls may span state changes |
| ICRC ledger/index | Ledger queries, index analytics, and archive callbacks | Index histories expose API exhaustion, not a stable snapshot version |
| ICRC tip certificate | Certificate and hash-tree evidence verified by the host adapter | Verification applies only when the ledger returns the required evidence |
| Cycle Minting Canister | Application-level certificate and hash-tree witness verified against the CMC and returned rate | Cycles per ICP is derived from the certified rate and the documented one-trillion-cycles-per-XDR protocol constant |
| Official IC Dashboard | Timestamped off-chain REST analytics | `certified: false` and `point_in_time_guaranteed: false` |

JSON reports keep raw identifiers, numeric fields, classifications, timestamps,
and explicit provenance. Text output may shorten or format values for people.
Report schemas are versioned independently and currently use versions `1` and
`2`; before 1.0, incompatible report shapes are hard cuts rather than
compatibility branches.

See [IC Reporting Adapters](https://github.com/dragginzgame/ic-query/blob/main/docs/design/ic-reporting-adapters.md) for the
authority model and follow-up query rules.

## Command families

```text
icq ic canister info|count|page
icq ic metrics <metric>
icq ic network boundary-node-data-centers
icq ic network daily-stats

icq nns registry version
icq nns subnet list|info|refresh
icq nns node list|info|refresh
icq nns node-provider list|info|refresh
icq nns node-operator list|info|refresh
icq nns data-center list|info|refresh
icq nns topology summary|coverage|versions|health|gaps|capacity|regions|providers|refresh
icq nns governance economics|metrics|reward-event|maturity-modulation
icq nns proposal list|info|refresh|cache
icq nns neuron list|info|refresh|cache

icq sns list|info|token|params|metrics|swap|upgrade
icq sns canister list
icq sns proposal list|info|refresh|cache
icq sns neuron list|refresh|cache

icq icrc ledger capabilities|token|index|transactions|block-types|archives|tip-certificate
icq icrc account balance|allowance
icq icrc account transaction page|list|refresh|cache

icq system xdr|cycles
```

The top-level `--network` option supplies network identity to NNS, SNS, and
system-canister commands. Built-in sources and caches currently accept only
the mainnet `ic` identity.

Dashboard canister and ICRC commands identify their target using a stable
entity id and an explicit API endpoint; Dashboard metric and network-resource
commands use an official resource identity and endpoint. These families reject
the global `--network` option; use the command’s `--source-endpoint` option when
an endpoint override is needed.

## Collection and cache behavior

Every data-producing command follows one documented collection mode:

| Mode | Network access | Cache writes |
| --- | --- | --- |
| Live query | Always | Never |
| Cache-backed, refresh if missing | Only when the complete cache is absent | Publishes a validated complete snapshot |
| Cache-preferred, live fallback | Only when cached data cannot satisfy the lookup | Only an explicit refresh writes complete collections |
| Local-only inspection | Never | Never |
| Forced refresh | Always | Atomically replaces the prior complete snapshot after validation |

Dashboard count, page, metric, daily-statistics, and boundary-node data-center
commands always make exactly one REST request. A page returns at most 100
canister summaries and never follows its cursors automatically. A metric query
defaults to one hour at a five-minute step and is capped at 1,000 observations
per series. Daily statistics default to seven days and are capped at one year
and 366 rows. The boundary-node report consumes one non-paginated data-center
resource and makes no per-location calls. None of these commands creates a
cache. Successful SNS metrics queries default to a 30-day proposal-count
window capped at 365 days. They make three targeted client requests, preserve
Governance-cached treasury and voting-power timestamps, and do not scan
transactions, fan out, or create a cache. Paged proposal, neuron, and
account-history collections retain refresh attempt state. Failed or capped
refreshes do not replace the last complete snapshot.
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

## Library

Use `ic-query` for typed requests, reports, validation, cache behavior, source
adapters, and renderers without spawning `icq`.

Pure DTO and rendering use has no host dependencies:

```toml
[dependencies]
ic-query = { version = "0.25", default-features = false }
```

Native tools that need live calls, filesystem caches, refreshes, or custom
source adapters enable `host`:

```toml
[dependencies]
ic-query = { version = "0.25", default-features = false, features = ["host"] }
```

The no-default build is checked for `wasm32-unknown-unknown` without Clap,
`ic-agent`, Reqwest, Tokio, or `futures`. This is a host-dependency boundary,
not a `no_std` promise.

Public report families are exposed from:

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
decoding, runtime bridge, and cache dependencies required by that API while
leaving `ic-query`'s direct optional Dashboard `reqwest` transport and
`serde_cbor` certification dependencies disabled. Because the feature still
includes `ic-agent`, both packages may remain in its transitive dependency
graph. The full `host` feature remains the choice for all reporting adapters
and is a strict superset.

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
