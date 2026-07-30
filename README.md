# ic-query

[![CI](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml/badge.svg)](https://github.com/dragginzgame/ic-query/actions/workflows/ci.yml)
[![docs.rs](https://docs.rs/ic-query/badge.svg)](https://docs.rs/ic-query)
[![License](https://img.shields.io/crates/l/ic-query.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.91.0-blue.svg)](Cargo.toml)

`ic-query` provides read-only Internet Computer metadata query code, and
`ic-query-cli` provides the `icq` executable wrapper.

`icq` currently supports NNS, SNS, and generic ICRC metadata queries: registry
version, subnet catalog lookup, node/provider/operator/data-center inventory,
topology reports, NNS proposals and publicly readable neuron views, deployed
SNS reports, and ICRC ledger capabilities, token, balance, allowance, index
discovery, ledger and account transaction history, block type, archive, and
tip certificate reports.

## Install

From this checkout:

```bash
make install
```

The local install target replaces an existing `icq` binary so repeated
development installs do not require a separate Cargo `--force` option.

From crates.io after publication:

```bash
cargo install ic-query-cli
```

## Library

Use `ic-query` for typed report models and renderers without the `icq` process
wrapper. The default feature set is empty:

```toml
[dependencies]
ic-query = { version = "0.15", default-features = false }
```

Feature boundary:

| Feature | Intended use |
| --- | --- |
| none / `default-features = false` | Pure request/report DTOs, text renderers, and local parsing/resolution helpers. CI checks this path for `wasm32-unknown-unknown` without host-only dependencies. |
| `host` | Native cache, refresh, live-call, and filesystem-backed report builders. Pulls in the native runtime/live-call dependencies. |

This is a host dependency boundary, not a `no_std` promise. No-default
builds may still use ordinary `std` types such as `String` and `Vec`.

Native tools that want live calls, cache-backed report builders, refresh
helpers, or custom source adapters enable `host`:

```toml
[dependencies]
ic-query = { version = "0.15", default-features = false, features = ["host"] }
```

Use `ic_query::icrc`, `ic_query::nns`, `ic_query::sns`, and
`ic_query::subnet_catalog` for the public report-family APIs. The `host`
feature also exposes source traits for fixture, mirror, proxy, or
pre-collected data sources. Native tools should normally depend on
`features = ["host"]`. Clap parsing, command dispatch, process output, and
project-context discovery belong exclusively to `ic-query-cli` and are not a
library feature.

Built-in host calls use one concrete adapter per authority family:
`ic_query::nns::LiveNnsSource`, `ic_query::sns::LiveSnsSource`, and
`ic_query::icrc::LiveIcrcSource`. Small report-specific capability traits keep
custom adapters narrow, while all NNS capability traits share
`ic_query::nns::NnsSourceRequest` for network and collection provenance.

Ordinary library builders and refresh functions are silent. Native consumers
that want live paged-refresh updates can use the matching `*_with_progress`
entry point and handle `QueryProgressEvent` values in their own presentation
layer. The `icq` executable supplies the stderr renderer; the reusable library
never selects a process output sink.

Each NNS and SNS family root is its sole public path. For example, topology
consumers use `ic_query::nns::topology::*`; internal `report` modules are not
public API.

See
[Library Usage](https://github.com/dragginzgame/ic-query/blob/main/docs/library-usage.md)
for downstream feature guidance, source-adapter examples, and patterns for
using request constructors and report builders instead of process shell-outs.

## Roadmap

The living [Roadmap to 1.0](docs/roadmap/1.0.md) tracks current reporting
coverage, prioritized NNS/SNS/ICRC and IC-wide workstreams, caching policy, and
the stability bar for 1.0. Adapter ownership and provenance rules remain in
[IC Reporting Adapters](docs/design/ic-reporting-adapters.md).

## Commands

```bash
icq nns help
icq nns registry version
icq nns subnet [list|info|refresh]
icq nns node [list|info|refresh]
icq nns node-provider [list|info|refresh]
icq nns node-operator [list|info|refresh]
icq nns data-center [list|info|refresh]
icq nns proposal [list|info|refresh|cache]
icq nns neuron [list|info|refresh|cache]
icq nns topology [summary|coverage|versions|health|gaps|capacity|regions|providers|refresh]
icq icrc ledger [capabilities|token|index|transactions|block-types|archives|tip-certificate]
icq icrc account [balance|allowance|transaction]
icq icrc account transaction [page|list|refresh|cache]
icq sns [list|info|token|params|proposal|neuron]
icq sns proposal [list|info|refresh|cache]
icq sns neuron [list|refresh|cache]
```

Use `icq nns <family> help`, `icq nns topology <report> help`, or
`icq icrc <family> <command> help`, or `icq sns <command> help` for command
options.
Use `icq -V` or `icq --version` for the executable version; command families do
not expose positional version shortcuts.

The top-level `--network` option supplies network identity to NNS and SNS
commands, including NNS proposals. ICRC commands identify their target by
ledger canister and API endpoint instead; combining `--network` with `icrc` is
rejected before dispatch and directs the caller to `--source-endpoint`.
The built-in NNS and SNS sources and caches currently support only the mainnet
`ic` identity, so another network name is rejected before family dispatch.

Most commands support text output by default and JSON output with
`--format json`:

```bash
icq --network ic nns subnet info ryjl3-tyaaa-aaaaa-aaaba-cai --format json
```

All current report `schema_version` values are `1`. Before 1.0, a hard-cut
shape replaces its predecessor instead of extending a historical schema
number sequence.

Generic ICRC ledgers can be queried directly by ledger canister id. Live
commands include the queried source endpoint in text and JSON reports and
support endpoint overrides with `--source-endpoint`:

```bash
icq icrc ledger capabilities mxzaz-hqaaa-aaaar-qaada-cai
icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai --format json
icq icrc account balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa
icq icrc account balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --subaccount 0000000000000000000000000000000000000000000000000000000000000000
icq icrc account allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa
icq icrc account allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa --owner-subaccount 0000000000000000000000000000000000000000000000000000000000000000 --spender-subaccount 0000000000000000000000000000000000000000000000000000000000000000
icq icrc account transaction page mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa
icq icrc account transaction page mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --start 12345 --limit 25 --format json
icq icrc account transaction page ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa --index-canister-id qhbym-qaaaa-aaaaa-aaafq-cai
icq icrc account transaction refresh mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa
icq icrc account transaction list mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa --sort oldest --limit 100
icq icrc account transaction cache status mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa
icq icrc ledger index ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc ledger index ryjl3-tyaaa-aaaaa-aaaba-cai --format json
icq icrc ledger transactions ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc ledger transactions ryjl3-tyaaa-aaaaa-aaaba-cai --start 100 --limit 50 --format json
icq icrc ledger transactions mxzaz-hqaaa-aaaar-qaada-cai --start 0 --limit 1 --follow-archives
icq icrc ledger block-types ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc ledger archives ryjl3-tyaaa-aaaaa-aaaba-cai --from qaa6y-5yaaa-aaaaa-aaafa-cai --format json
icq icrc ledger tip-certificate mxzaz-hqaaa-aaaar-qaada-cai
```

Account-history page and refresh operations discover the index through
ICRC-106 unless `--index-canister-id` is supplied, then verify that the
selected index reports the requested ledger. Reports retain ledger, index,
account, endpoint, and pagination provenance. Pass the page report's
`next_start` value back through `--start` to fetch the next older page. Cursors
are arbitrary-size unsigned decimal Candid `Nat` values. The ICP ledger does
not export ICRC-106, so its official index must be supplied explicitly as
shown above. Both generic ICRC index-ng and deployed ICP index transactions
are supported.

`transaction refresh` exhausts the verified index and publishes one complete
account cache atomically. `transaction list` and `transaction cache status`
are strictly local and never make a network request. Cache identity includes
the source endpoint, ledger, account owner, and subaccount; page size, list
limit, and sort remain operation or view options. A failed or capped refresh
records attempt progress, including the resolved index canister when
available, and leaves the prior complete cache unchanged. Custom collection
sources must return the explicitly requested index canister when one was
provided. Local list projection consumes the loaded transaction rows directly
instead of cloning the complete snapshot before applying sort and limit.
Because the index interface exposes no snapshot version, complete caches prove
API exhaustion but explicitly report `point_in_time_guaranteed: false`.

When `icrc3_get_tip_certificate` returns evidence, the built-in live source
authenticates the IC certificate, including delegation, canister authority,
and freshness, then proves that the returned hash tree matches
`certified_data` and contains canonical `last_block_index` and
`last_block_hash` leaves. Endpoint overrides must therefore serve
mainnet-certified responses; alternate root keys are not configured.

## Cache

Detailed command help identifies one of five collection modes:

- live queries do not read or write a report cache;
- cache-backed reads refresh and store a complete snapshot only when missing;
- cache-preferred reads use a complete snapshot when available and otherwise
  make a live query;
- local cache inspection never makes a network request;
- forced refreshes fetch and validate a complete snapshot before any atomic
  cache replacement.

SNS neuron list mode is view-dependent: `--sort api` is a bounded live query,
while other sorts require a complete local snapshot.

The CLI uses one user-level cache root for every working directory. It resolves
the root from the first non-empty source below:

1. `ICQ_CACHE_ROOT` (an explicit absolute override);
2. `$XDG_CACHE_HOME/ic-query`; or
3. `$HOME/.cache/ic-query`.

It does not inspect the current directory for `icp.yaml` or `dfx.json`, and it
does not read or migrate former project-local `.icq` directories. Library cache
request types take this actual cache root; they do not append another `.icq`
directory.

The NNS subnet, node, provider, operator, data-center, and topology commands
share this cache. Refresh commands fetch current mainnet registry data and
replace the matching cache atomically:

```bash
icq nns subnet refresh
icq nns topology refresh
```

List/info commands populate their component cache on first use and print the
API endpoint they are calling before creating it. Refresh commands force a
fresh fetch and replace the matching cache.

Complete NNS and SNS proposal/neuron snapshots likewise use cache schema
version 1 and require `domain`, `entity`, `collection`, and `scope` identity
fields. Snapshot files that do not match the current shape are unsupported and must be
refreshed; there is no version bridge or migration path. Snapshot row counts,
required row ids, uniqueness, and embedded identity are validated when loaded.
Refresh locks and attempt sidecars accept only their exact current fields and
validate schema, network, identity, and lifecycle state. Stale or malformed
locks are reported but never deleted automatically; remove one manually only
after verifying that no refresh is still running.

Complete ICRC account-history snapshots use the same atomic snapshot, lock,
and refresh-attempt lifecycle under
`<cache-root>/icrc/ic/account-<identity-hash>/transactions/full.json`. They require an
explicit `transaction refresh`; normal cache-only list and status operations
do not start a potentially long index crawl. The public library additionally
offers distinct refresh-if-missing and refresh-if-stale policies for consumers
that choose those behaviors explicitly.

SNS neuron commands keep quick `--sort api` output on a bounded live query.
Whole-collection neuron sorts use complete snapshots and require an explicit
refresh first:

```bash
icq sns neuron refresh 1
icq sns neuron list 1 --limit 500 --sort stake
```

Complete SNS neuron snapshots live under
`<cache-root>/sns/ic/<root-principal>/neurons/full.json`. Failed or capped refresh
attempts are recorded separately and do not replace the last complete snapshot.
Refresh shows a same-line stderr progress counter with pages and rows fetched
when running in a terminal. If the complete snapshot is published but final
attempt metadata cannot be written, the refresh remains successful and reports
the sidecar finalization error explicitly.

Inspect local SNS neuron snapshots and their latest refresh-attempt metadata
without making live calls:

```bash
icq sns neuron cache list
icq sns neuron cache status 1
```

Cache list and status commands are local-only; malformed, unsupported, or
identity-mismatched snapshot files are shown as invalid local cache rows.

Numeric cache lookup scans snapshot headers and loads only the matching
complete snapshot. If duplicate caches claim an id, use the root principal to
select the intended cache explicitly.

Live API neuron listings are capped at 100 rows per call. Cache-backed sorts
can use larger `--limit` values because they read from the complete local
snapshot.

Neuron IDs are shortened to eight characters in text tables by default. Use
`icq sns neuron list 1 --verbose` to show full neuron IDs.
Text output shows current SNS token amounts, including token fee, total supply,
stake, maturity, and staked maturity, as token decimals with two places. JSON
keeps the raw base-unit and e8s fields. ICRC metadata values, including token
logos, also remain raw in JSON; text reports show only logo presence.

SNS governance nervous system parameters can be queried by list id or root
principal:

```bash
icq sns params 1
icq sns params 23ten-uaaaa-aaaaq-aabia-cai --format json
```

NNS governance proposals can be queried from the mainnet NNS governance
canister. Without a complete local snapshot, list views are bounded live
queries; status filters are sent to governance where supported, topic filters
are applied to returned rows, query filters search returned title, action,
summary, and URL text, and local sort modes mirror the SNS proposal direction
rules. Text and JSON list reports include `result_scope` so bounded live views
are distinguishable from complete-cache views:

```bash
icq nns proposal list --limit 25
icq nns proposal list --status open
icq nns proposal list --reward-status settled
icq nns proposal list --topic governance
icq nns proposal list --proposer 123456789
icq nns proposal list --query subnet
icq nns proposal list --sort reward-status
icq nns proposal list --sort tally-time
icq nns proposal list --sort deadline
icq nns proposal list --sort voting-power
icq nns proposal list --sort proposed
icq nns proposal list --sort title --asc
icq nns proposal info 132411
icq nns proposal info 132411 --ballots
icq nns proposal info 132411 --verbose
icq nns proposal info 132411 --format json
```

NNS proposal list views support
`--proposer <neuron-id>`, `--query <text>`, and
`--sort api|id|status|reward-status|topic|proposer|title|action|yes|no|total-votes|tally-time|voting-power|ballots|reject-cost|reward-round|proposed|deadline|decided|executed|failed`.
Local sort modes accept `--asc` or `--desc`; status, reward status, topic,
proposer, title, and action default to ascending, while id, tally values, tally
time, ballot count, reject cost, reward round, voting power, and timestamp
sorts default to descending.

Complete NNS proposal snapshots can be refreshed and inspected explicitly. A
refresh pages through NNS governance until the API is exhausted, writes progress
to stderr in a terminal, and publishes only complete snapshots:

```bash
icq nns proposal refresh
icq nns proposal refresh --max-pages 5
icq nns proposal cache list
icq nns proposal cache status
```

Complete NNS proposal snapshots live under
`<cache-root>/nns/ic/governance/proposals/full.json`. Failed or capped refresh attempts
are recorded separately and do not replace the last complete snapshot. Proposal
list and detail lookups reuse an existing complete snapshot when it can satisfy
the request, then fall back to live governance lookup.
Cache list and status commands are local-only; malformed, unsupported, or
identity-mismatched snapshot files are shown as invalid local cache rows.

Publicly readable NNS neuron views come directly from the mainnet Governance
canister. They preserve raw state, visibility, neuron-type, and vote values
alongside stake, staked maturity, voting power, dissolve state, known-neuron
metadata, and recent ballots. They do not expose authenticated owner state
such as controllers, followees, or private unstaked maturity:

```bash
icq nns neuron list --limit 25
icq nns neuron list --start-neuron-id 123456789 --format json
icq nns neuron info 123456789 --verbose
icq nns neuron refresh
icq nns neuron cache status
```

List and detail reads prefer
`<cache-root>/nns/ic/governance/neurons/full.json` when a valid complete
snapshot can satisfy the request, then fall back to a bounded live query.
Only `neuron refresh` writes the snapshot; `neuron cache status` is local-only.
Refresh walks the ascending Governance index through API exhaustion under one
lock and publishes atomically. Because Governance exposes no stable version
for the collection, reports and caches state
`point_in_time_guaranteed: false`.

SNS governance proposals can be queried as cached list views or direct live
detail lookups. Normal proposal list views auto-create a complete local
snapshot on first use, then apply supported view options locally. Proposal
detail lookups reuse an existing complete local snapshot when it contains the
requested proposal, then fall back to live detail lookup. Status and topic
filters that can be reproduced from complete proposal rows use the local
snapshot, including decided/adopted/rejected status filters; reward eligibility
can be filtered with `--eligible any|yes|no`, and proposer neuron ids can be
filtered by prefix with `--proposer`. Use `--query <text>` to search proposal
title, action, summary, URL, and payload text:

```bash
icq sns proposal list 1 --limit 25
icq sns proposal list 1 --status open
icq sns proposal list 1 --status decided
icq sns proposal list 1 --eligible yes
icq sns proposal list 1 --eligible no
icq sns proposal list 1 --proposer 00010203
icq sns proposal list 1 --query treasury
icq sns proposal list 1 --sort status
icq sns proposal list 1 --sort topic
icq sns proposal list 1 --sort proposer
icq sns proposal list 1 --sort title
icq sns proposal list 1 --sort title --desc
icq sns proposal list 1 --sort action
icq sns proposal list 1 --sort action-id
icq sns proposal list 1 --sort total-votes
icq sns proposal list 1 --sort tally-time
icq sns proposal list 1 --sort ballots
icq sns proposal list 1 --sort eligible
icq sns proposal list 1 --sort reject-cost
icq sns proposal list 1 --sort reward-round
icq sns proposal list 1 --sort reward-end
icq sns proposal list 1 --sort created
icq sns proposal list 1 --sort decided
icq sns proposal list 1 --sort executed
icq sns proposal list 1 --sort failed
icq sns proposal list 1 --sort created --asc
icq sns proposal list 1 --topic governance
icq sns proposal list 1 --status decided --topic governance
icq sns proposal list 1 --before 100 --format json
icq sns proposal info 1 387
icq sns proposal info 1 387 --ballots
```

Proposal list views support
`--eligible any|yes|no`, `--proposer <neuron-id-prefix>`, `--query <text>`, and
`--sort api|id|status|topic|proposer|title|action|action-id|yes|no|total-votes|tally-time|ballots|eligible|reject-cost|reward-round|reward-end|created|decided|executed|failed`.
Local sort modes accept `--asc` or `--desc`; status, topic, proposer, title,
and action default to ascending, while id, action id, tally values, tally time,
ballot count, reward eligibility, reject cost, reward round, and timestamp
sorts default to descending. Cache-backed views filter and sort complete
local snapshots before applying `--limit`.

Complete SNS proposal snapshots can also be refreshed and inspected manually:

```bash
icq sns proposal refresh 1
icq sns proposal cache list
icq sns proposal cache status 1
```

Cache list and status commands are local-only; malformed, unsupported, or
identity-mismatched snapshot files are shown as invalid local cache rows.
Numeric cache lookup scans snapshot headers and loads only the matching
complete snapshot. If duplicate caches claim an id, use the root principal to
select the intended cache explicitly.

## Integration

`icq` is a standalone metadata lookup tool. Orchestration, deployment, and
application repositories can call the CLI when they need IC metadata instead of
linking registry adapters directly. For one integration example, see
[Canic](https://github.com/dragginzgame/canic).

## Status

The command namespace is intentionally small:

- `nns` is implemented.
- `nns proposal list` and `nns proposal info` are cache-aware mainnet NNS
  governance proposal queries: they reuse complete local snapshots when those
  snapshots can satisfy the request, then fall back to bounded or direct live
  governance queries where applicable.
- `nns proposal refresh` caches complete mainnet NNS governance proposal
  snapshots.
- `nns proposal cache list|status` inspects local complete NNS proposal
  snapshots and refresh-attempt metadata without live calls.
- `nns neuron list|info` provides cache-preferred publicly readable mainnet
  Governance neuron views.
- `nns neuron refresh` atomically caches a complete ordered public neuron-index
  walk, and `nns neuron cache status` inspects it without a live call.
- `sns list`, `sns info`, `sns token`, `sns params`, `sns proposal`,
  `sns proposals`, and `sns neurons` are implemented for deployed mainnet SNS
  instances.
- `sns proposals` auto-creates and reuses complete proposal snapshots for
  cache-backed list views.
- `sns proposals refresh` force-refreshes complete proposal snapshots.
- `sns proposals cache list|status` inspects local complete proposal snapshots
  and refresh-attempt metadata without live calls.
- `sns neurons refresh` caches complete neuron snapshots for cache-backed
  sorting.
- `sns neurons cache list|status` inspects local complete neuron snapshots and
  refresh-attempt metadata without live calls.
- Additional IC query families can be added without coupling query code to
  deployment tooling.
