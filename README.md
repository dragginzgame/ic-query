# ic-query

[![CI](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml/badge.svg)](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml)
[![docs.rs](https://docs.rs/ic-query/badge.svg)](https://docs.rs/ic-query)
[![License](https://img.shields.io/crates/l/ic-query.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.91.0-blue.svg)](Cargo.toml)

`ic-query` is a read-only Internet Computer reporting library.
`ic-query-cli` provides its `icq` command-line interface.

The project turns Registry, NNS, SNS, ledger/index, certificate, and official
IC Dashboard responses into typed reports with explicit provenance. It keeps
live calls, cache reads, refreshes, and local-only inspection visibly distinct.

## Supported reporting

| Family | Current surface |
| --- | --- |
| Official IC Dashboard | Bounded canister count/search pages, deployed canister metadata and upgrade history, and bounded network metric time series |
| NNS Registry | Registry version, Subnets, nodes, node operators, node providers, data centers, component topology diagnostics, and an exact-version joined topology library API |
| NNS Governance | Proposals, publicly readable neurons, economics, metrics, latest reward event, and maturity modulation |
| SNS | Discovery, metadata, token and nervous-system parameters, Root canister inventory and health, proposals, and neurons |
| ICRC | Capabilities, token metadata, balances, allowances, index discovery, ledger and account transactions, archives, block types, and tip certificates |

The living [Roadmap to 1.0](docs/roadmap/1.0.md) records the broader reporting
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
icq ic metrics ic-node-count --format json

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
icq sns proposal list 1 --limit 25

# Generic ICRC reports
icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc account balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa
```

Text is the default human-facing format. Use `--format json` on report commands
for raw, script-friendly fields:

```bash
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --format json
icq nns topology summary --format json
```

Run `icq help`, `icq <family> help`, or append `help` to any nested command for
its current options and collection mode. The complete command map and cache
behavior are documented in [CLI Usage](docs/cli-usage.md).

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
| Official IC Dashboard | Timestamped off-chain REST analytics | `certified: false` and `point_in_time_guaranteed: false` |

JSON reports keep raw identifiers, numeric fields, classifications, timestamps,
and explicit provenance. Text output may shorten or format values for people.
All current report `schema_version` values are `1`; before 1.0, incompatible
report shapes are hard cuts rather than compatibility branches.

See [IC Reporting Adapters](docs/design/ic-reporting-adapters.md) for the
authority model and follow-up query rules.

## Command families

```text
icq ic canister info|count|page
icq ic metrics <metric>

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

icq sns list|info|token|params
icq sns canister list
icq sns proposal list|info|refresh|cache
icq sns neuron list|refresh|cache

icq icrc ledger capabilities|token|index|transactions|block-types|archives|tip-certificate
icq icrc account balance|allowance
icq icrc account transaction page|list|refresh|cache
```

The top-level `--network` option supplies network identity to NNS and SNS
commands. Built-in sources and caches currently accept only the mainnet `ic`
identity.

Dashboard canister and ICRC commands identify their target using a stable
entity id and an explicit API endpoint; Dashboard metrics use an official
metric name, explicit time bounds, and an endpoint. These families reject the
global `--network` option; use the command’s `--source-endpoint` option when an
endpoint override is needed.

## Collection and cache behavior

Every data-producing command follows one documented collection mode:

| Mode | Network access | Cache writes |
| --- | --- | --- |
| Live query | Always | Never |
| Cache-backed, refresh if missing | Only when the complete cache is absent | Publishes a validated complete snapshot |
| Cache-preferred, live fallback | Only when cached data cannot satisfy the lookup | Only an explicit refresh writes complete collections |
| Local-only inspection | Never | Never |
| Forced refresh | Always | Atomically replaces the prior complete snapshot after validation |

Dashboard count, page, and metric commands always make exactly one REST
request. A page returns at most 100 canister summaries and never follows its
cursors automatically. A metric query defaults to one hour at a five-minute
step and is capped at 1,000 observations per series. None of these commands
creates a cache. Paged proposal, neuron, and account-history collections retain
refresh attempt state. Failed or capped refreshes do not replace the last
complete snapshot.
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
[Cache Policy](docs/design/cache-policy.md).

## Library

Use `ic-query` for typed requests, reports, validation, cache behavior, source
adapters, and renderers without spawning `icq`.

Pure DTO and rendering use has no host dependencies:

```toml
[dependencies]
ic-query = { version = "0.20", default-features = false }
```

Native tools that need live calls, filesystem caches, refreshes, or custom
source adapters enable `host`:

```toml
[dependencies]
ic-query = { version = "0.20", default-features = false, features = ["host"] }
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

Built-in host calls use one adapter per authority family:
`LiveIcSource`, `LiveIcrcSource`, `LiveNnsSource`, and `LiveSnsSource`.
Report-specific capability traits let fixtures, mirrors, proxies, and
pre-collected sources reuse the same validation and projection path.

Library builders do not write to stdout or stderr. Paged refresh APIs can emit
typed `QueryProgressEvent` values to a caller-provided sink; terminal rendering
remains an `ic-query-cli` responsibility.

See [Library Usage](docs/library-usage.md) for complete examples and feature
guidance.

## Documentation

- [Documentation index](docs/README.md)
- [CLI usage and collection modes](docs/cli-usage.md)
- [Library usage](docs/library-usage.md)
- [Roadmap to 1.0](docs/roadmap/1.0.md)
- [IC Dashboard canister reporting](docs/design/ic-dashboard-canister-reporting.md)
- [IC Dashboard network metrics](docs/design/ic-dashboard-network-metrics.md)
- [Exact-version NNS Subnet topology](docs/design/nns-subnet-topology.md)
- [SNS Root canister inventory and health](docs/design/sns-root-canister-reporting.md)
- [Release ledger](CHANGELOG.md)

## Scope

`ic-query` is read-only metadata and evidence tooling. It does not replace
`dfx`, expose arbitrary Candid invocation, or perform canister, governance,
ledger, or network mutations.

Deployment and orchestration projects can call the CLI or use the Rust library
when they need these reporting contracts. One downstream integration is
[Canic](https://github.com/dragginzgame/canic).
