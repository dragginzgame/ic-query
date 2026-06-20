# ic-query

[![CI](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml/badge.svg)](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml)
[![docs.rs](https://docs.rs/ic-query/badge.svg)](https://docs.rs/ic-query)
[![License](https://img.shields.io/crates/l/ic-query.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.91.0-blue.svg)](Cargo.toml)

`ic-query` provides the `icq` executable for read-only Internet Computer
metadata queries.

`icq` currently supports NNS and SNS metadata queries: registry version,
subnet catalog lookup, node/provider/operator/data-center inventory, topology
reports, and deployed SNS reports.

## Install

From this checkout:

```sh
make install
```

From crates.io after publication:

```sh
cargo install ic-query
```

## Commands

```sh
icq nns help
icq nns registry version
icq nns subnet [list|info|refresh]
icq nns node [list|info|refresh]
icq nns node-provider [list|info|refresh]
icq nns node-operator [list|info|refresh]
icq nns data-center [list|info|refresh]
icq nns proposal [list|info|refresh|cache]
icq nns topology [summary|coverage|versions|health|gaps|capacity|regions|providers|refresh]
icq sns [list|info|token|params|proposal|proposals|neurons]
icq sns proposals [cache|refresh]
icq sns neurons [cache|refresh]
```

Use `icq nns <family> help`, `icq nns topology <report> help`, or
`icq sns <command> help` for command options.

Most commands support text output by default and JSON output with
`--format json`:

```sh
icq --network ic nns subnet info ryjl3-tyaaa-aaaaa-aaaba-cai --format json
```

## Cache

The NNS subnet, node, provider, operator, data-center, and topology commands
use project-local cache files under `.icq/`. Refresh commands fetch current
mainnet registry data and replace the matching cache atomically:

```sh
icq nns subnet refresh
icq nns topology refresh
```

List/info commands populate their component cache on first use and print the
API endpoint they are calling before creating it. Refresh commands force a
fresh fetch and replace the matching cache.

SNS neuron commands keep quick `--sort api` output on a bounded live query.
Whole-collection neuron sorts use complete snapshots and require an explicit
refresh first:

```sh
icq sns neurons refresh 1
icq sns neurons 1 --limit 500 --sort stake
```

Complete SNS neuron snapshots live under
`.icq/sns/ic/<root-principal>/neurons/full.json`. Failed or capped refresh
attempts are recorded separately and do not replace the last complete snapshot.
Refresh shows a same-line stderr progress counter with pages and rows fetched
when running in a terminal.

Inspect local SNS neuron snapshots and their latest refresh-attempt metadata
without making live calls:

```sh
icq sns neurons cache list
icq sns neurons cache status 1
```

Live API neuron listings are capped at 100 rows per call. Cache-backed sorts
can use larger `--limit` values because they read from the complete local
snapshot.

Neuron IDs are shortened to eight characters in text tables by default. Use
`icq sns neurons 1 --verbose` to show full neuron IDs.
Text output shows current SNS token amounts, including token fee, total supply,
stake, maturity, and staked maturity, as token decimals with two places. JSON
keeps the raw base-unit and e8s fields.

SNS governance nervous system parameters can be queried by list id or root
principal:

```sh
icq sns params 1
icq sns params 23ten-uaaaa-aaaaq-aabia-cai --format json
```

NNS governance proposals can be queried from the mainnet NNS governance
canister. Without a complete local snapshot, list views are bounded live
queries; status filters are sent to governance where supported, topic filters
are applied to returned rows, and local sort modes mirror the SNS proposal
direction rules:

```sh
icq nns proposal list --limit 25
icq nns proposal list --status open
icq nns proposal list --reward-status settled
icq nns proposal list --topic governance
icq nns proposal list --sort proposed
icq nns proposal list --sort title --asc
icq nns proposal info 132411
icq nns proposal info 132411 --ballots
icq nns proposal info 132411 --verbose
icq nns proposal info 132411 --format json
```

NNS proposal list views support
`--sort api|id|status|topic|proposer|title|action|yes|no|total-votes|ballots|reject-cost|reward-round|proposed|decided|executed|failed`.
Local sort modes accept `--asc` or `--desc`; status, topic, proposer, title,
and action default to ascending, while id, tally, ballot count, reject cost,
reward round, and timestamp sorts default to descending.

Complete NNS proposal snapshots can be refreshed and inspected explicitly. A
refresh pages through NNS governance until the API is exhausted, writes progress
to stderr in a terminal, and publishes only complete snapshots:

```sh
icq nns proposal refresh
icq nns proposal refresh --max-pages 5
icq nns proposal cache list
icq nns proposal cache status
```

Complete NNS proposal snapshots live under
`.icq/nns/ic/governance/proposals/full.json`. Failed or capped refresh attempts
are recorded separately and do not replace the last complete snapshot. Proposal
list and detail lookups reuse an existing complete snapshot when it can satisfy
the request, then fall back to live governance lookup.

SNS governance proposals can be queried as cached list views or direct live
detail lookups. Normal proposal list views auto-create a complete local
snapshot on first use, then apply supported view options locally. Proposal
detail lookups reuse an existing complete local snapshot when it contains the
requested proposal, then fall back to live detail lookup. Topic filters and
adopted/rejected status filters currently use bounded live queries:

```sh
icq sns proposals 1 --limit 25
icq sns proposals 1 --status open
icq sns proposals 1 --status decided
icq sns proposals 1 --sort status
icq sns proposals 1 --sort proposer
icq sns proposals 1 --sort title
icq sns proposals 1 --sort title --desc
icq sns proposals 1 --sort action
icq sns proposals 1 --sort total-votes
icq sns proposals 1 --sort ballots
icq sns proposals 1 --sort reject-cost
icq sns proposals 1 --sort reward-round
icq sns proposals 1 --sort created
icq sns proposals 1 --sort decided
icq sns proposals 1 --sort executed
icq sns proposals 1 --sort failed
icq sns proposals 1 --sort created --asc
icq sns proposals 1 --topic governance
icq sns proposals 1 --before 100 --format json
icq sns proposal 1 387
icq sns proposal 1 387 --ballots
```

Proposal list views support
`--sort api|id|status|proposer|title|action|yes|no|total-votes|ballots|reject-cost|reward-round|created|decided|executed|failed`.
Local sort modes accept `--asc` or `--desc`; status, proposer, title, and
action default to ascending, while id, tally, ballot count, reject cost,
reward round, and timestamp sorts default to descending. Cache-compatible
views filter and sort complete local snapshots before applying `--limit`;
bounded live fallback views sort the returned API rows.

Complete SNS proposal snapshots can also be refreshed and inspected manually:

```sh
icq sns proposals refresh 1
icq sns proposals cache list
icq sns proposals cache status 1
```

## Integration

`icq` is a standalone metadata lookup tool. Orchestration, deployment, and
application repositories can call the CLI when they need IC metadata instead of
linking registry adapters directly. For one integration example, see
[Canic](https://github.com/dragginzgame/canic).

## Status

The command namespace is intentionally small:

- `nns` is implemented.
- `nns proposal list` and `nns proposal info` are implemented as direct live
  mainnet NNS governance proposal queries.
- `nns proposal refresh` caches complete mainnet NNS governance proposal
  snapshots.
- `nns proposal cache list|status` inspects local complete NNS proposal
  snapshots and refresh-attempt metadata without live calls.
- `sns list`, `sns info`, `sns token`, `sns params`, `sns proposal`,
  `sns proposals`, and `sns neurons` are implemented for deployed mainnet SNS
  instances.
- `sns proposals` auto-creates and reuses complete proposal snapshots for
  cache-compatible list views.
- `sns proposals refresh` force-refreshes complete proposal snapshots.
- `sns proposals cache list|status` inspects local complete proposal snapshots
  and refresh-attempt metadata without live calls.
- `sns neurons refresh` caches complete neuron snapshots for cache-backed
  sorting.
- `sns neurons cache list|status` inspects local complete neuron snapshots and
  refresh-attempt metadata without live calls.
- Additional IC query families can be added without coupling query code to
  deployment tooling.
