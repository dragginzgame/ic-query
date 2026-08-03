# CLI Usage

`icq` exposes read-only reports grouped by the authority or protocol family
that supplies them. This guide describes the stable command hierarchy and
collection behavior. Run a command with `--help`, or use `icq help <path>`, for
its complete current option reference. A command namespace without its next
operation displays the same complete local help as its explicit `help`
subcommand. Incomplete leaf operations and invalid values remain errors. Every
`Commands` section is ordered alphabetically by command name, including the
generated `help` entry.

## Common behavior

```bash
icq help
icq help nns
icq sns
icq sns reward
icq nns topology summary --help
```

Report commands use human-facing text by default and accept
`--json` for raw, script-friendly output. JSON reports retain stable
identifiers, native numeric fields, source endpoints, collection timestamps,
and authority guarantees.

Report schemas are versioned independently and currently use `schema_version`
values `1` and `2`. Before 1.0, incompatible shape changes replace the previous
contract without aliases or legacy readers.

## Target identity

NNS, SNS, and system-canister commands accept the global network identity:

```bash
icq --network ic nns registry version
icq --network ic sns list
icq --network ic system xdr
```

The built-in sources and caches currently support only mainnet, named `ic`.
A different network is rejected before a live adapter is constructed.

Official Dashboard canister and ICRC commands identify their target using an
entity principal plus `--source-endpoint`. Dashboard metric and network
resource commands use an official resource identity and `--source-endpoint`.
These families reject the top-level `--network` option instead of silently
ignoring it.

## Official IC Dashboard

```bash
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --json
icq ic canister count --has-name true
icq ic canister page --query ledger --limit 25 --json
icq ic canister page --after ryjl3-tyaaa-aaaaa-aaaba-cai --limit 25
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai \
  --source-endpoint https://ic-api.internetcomputer.org/api/v3

icq ic metrics instruction-rate
icq ic metrics cycle-burn-rate \
  --start 1700000000 --end 1700003600 --step 300
icq ic metrics ic-node-count --json

icq ic network boundary-node-data-centers
icq ic network boundary-node-data-centers --json
icq ic network daily-stats
icq ic network daily-stats \
  --start 1784937600 --end 1785542400
```

`info` preserves the Dashboard canister id, raw optional classification, name,
Subnet, controllers, language, module hash, update timestamp, and nullable
proposal-linked upgrade history. `count` applies the official Dashboard
filters and returns only the matching total. `page` returns canister-id-ordered
discovery rows plus the raw preceding/following cursors; use a discovered
canister principal with `info` for proposal-linked upgrade history.

Each command is a bounded live REST lookup that makes exactly one request. A
page defaults to 50 rows and is capped at the official API maximum of 100.
Cursors are followed only when a user explicitly supplies `--after` or
`--before` to another command. `metrics` selects one documented aggregate
network metric and preserves its raw named series, Unix timestamps, and
value strings. Its default window is the preceding hour at a
five-minute step; explicit windows are capped at 1,000 observations per
series. It does not fan out over metrics or Subnets.

`network boundary-node-data-centers` returns the official v4 data-center
aggregates in canonical id order. It preserves raw owner, region, coordinate,
and node-count strings, including zero-node locations, and derives only row and
node totals. The endpoint has no pagination parameters; the command makes one
request and no per-location follow-up calls.

`network daily-stats` selects the official v3 resource's daily network-activity
fields: raw average and maximum query, update, and total transaction rates plus
average block rate. It defaults to the preceding seven days and is capped at a
366-day window and 366 returned rows. Missing days remain missing; the command
makes one request and does not copy the resource's unrelated governance,
supply, topology, or Internet Identity fields into this report.

There is no automatic enumeration and these commands never read or write a
cache. The official Dashboard is an off-chain analytics authority, so every
report states
`certified: false` and `point_in_time_guaranteed: false`. It does not inherit a
Registry version. Canister reports also do not prove current controller or
module state.

See [IC Dashboard Canister Reporting](design/ic-dashboard-canister-reporting.md)
and [IC Dashboard Network Metrics](design/ic-dashboard-network-metrics.md) for
the canister and metric contracts. See
[IC Dashboard Boundary-Node Reporting](design/ic-dashboard-boundary-node-reporting.md)
and [IC Dashboard Daily Statistics](design/ic-dashboard-daily-stats.md) for the
network-resource contracts.

## NNS

### Registry inventory and topology

```bash
icq nns registry version

icq nns subnet list
icq nns subnet info <subnet-or-canister>
icq nns subnet refresh

icq nns node list
icq nns node-provider list
icq nns node-operator list
icq nns data-center list

icq nns topology refresh
icq nns topology summary
icq nns topology coverage
icq nns topology health
icq nns topology gaps
icq nns topology capacity
icq nns topology regions
icq nns topology providers
icq nns topology versions
```

Inventory list/detail commands use their documented cache-backed behavior.
Refresh commands force a live fetch and replace the matching complete cache
only after validation.

The CLI topology reports are diagnostics built from component caches. Their
version report makes component skew visible, but these reports are not
placement authority and do not claim that every input came from one Registry
version.

The host library separately exposes an exact-version joined Subnet topology
API. Its refresh resolves one Registry version and reads every Subnet, node,
operator, and provider relation at that version under one lock, then publishes
one atomic complete snapshot. It has no CLI surface today. Placement-sensitive
consumers should use that API instead of joining component caches; see
[Exact-Version NNS Subnet Topology](design/nns-subnet-topology.md).

### Governance

Bounded point-value reports are live-only:

```bash
icq nns governance economics
icq nns governance metrics --json
icq nns governance reward-event
icq nns governance maturity-modulation
```

The reward-event command returns the latest event, not complete reward
history.

Proposal and public-neuron collections expose live, cache-preferred, refresh,
and local inspection paths:

```bash
icq nns proposal list --limit 25
icq nns proposal info 132411
icq nns proposal refresh
icq nns proposal cache status

icq nns neuron list --limit 25
icq nns neuron info 123456789 --verbose
icq nns neuron refresh
icq nns neuron cache status
```

Complete refreshes page until API exhaustion and publish atomically. Governance
does not expose a stable collection version, so complete proposal and neuron
snapshots explicitly do not claim one point-in-time view. Public neuron
reports do not expose authenticated owner-only state.

## System canisters

Cycle Minting Canister reports are bounded live point queries:

```bash
icq system xdr
icq system xdr --json
icq system cycles
```

Both commands make exactly one native mainnet CMC
`get_icp_xdr_conversion_rate` query. The host adapter authenticates the IC
system certificate against the CMC principal, verifies that the returned hash
tree is committed by the canister's `certified_data`, and proves that the
native `ICP_XDR_CONVERSION_RATE` leaf equals the returned Candid value.

`xdr` preserves the raw market-data timestamp and
`xdr_permyriad_per_icp`. `cycles` preserves that same certified input and
derives `cycles_per_icp` exactly as
`xdr_permyriad_per_icp * 1_000_000_000_000 / 10_000`, using the IC protocol
constant of one trillion cycles per XDR. The report carries the formula and
raw certificate/hash-tree evidence; text additionally formats the rate to four
decimal places without replacing the raw field.

The CMC public Candid interface does not expose total cycles minted. `icq`
does not scrape the CMC's explicitly uncertified Prometheus metrics, call
hidden methods, enumerate canisters, or create a cache. See
[Certified CMC System Reporting](design/cmc-system-reporting.md).

## SNS

Resolve a deployed SNS by list id or Root principal:

```bash
icq sns list
icq sns info 1
icq sns token 1
icq sns parameters 1
icq sns metrics 1
icq sns metrics 23ten-uaaaa-aaaaq-aabia-cai --window 90d --json
icq sns swap 1
icq sns upgrade 1
```

`sns metrics` calls the Governance `get_metrics` composite query for the
resolved SNS. Its recent submitted/executed proposal window accepts nonzero
integer seconds, minutes, hours, or days, defaults to 30 days, and is capped
at 365 days before any live access. The report also preserves the latest
SNS-ledger block timestamp observed by Governance, genesis time, cached
treasury ledger/accounts and current/original e8s amounts with their own
timestamps, and cached voting-power metrics.

Including targeted discovery, the client makes three requests: one SNS-W
inventory query, one Governance metadata query for the resolved SNS, and the
metrics query. Governance performs its own bounded latest-block lookup inside
the composite query. The differently timestamped values do not form one
point-in-time snapshot. `icq` does not enumerate ledger transactions, fan out
to another SNS, or create a cache. See
[Bounded SNS Governance Metrics](design/0.24/0.24-design.md) for the complete
authority and validation contract.

`sns swap` makes exactly three bounded native swap queries for the resolved
SNS: `get_lifecycle`, `get_sale_parameters`, and `get_derived_state`. It keeps
component failures as typed gaps, does not claim the sequential responses are
one point-in-time snapshot, and never calls the potentially large
`get_state`, enumerates participants, or creates a cache. Including targeted
discovery, the command makes five canister queries: one SNS-W inventory query,
one Governance metadata query for the resolved SNS, and the three swap
queries. It does not enrich metadata for every deployed SNS.

`sns upgrade` makes two bounded report-specific queries: Governance
`get_running_sns_version` and SNS-W `get_next_sns_version`. Including targeted
discovery, it makes at most four canister queries. It preserves the native
six-role Wasm hashes and pending-upgrade state, and distinguishes a successful
response with no blessed successor from a failed next-version query. It does
not read the upgrade journal, download Wasms, fan out, create a cache, or claim
that the sequential responses form one point-in-time snapshot.

Inspect Root membership and read-only operational health:

```bash
icq sns canister list 1
icq sns canister list 23ten-uaaaa-aaaaq-aabia-cai --json
```

Inventory comes from `list_sns_canisters`. Health comes from
`get_sns_canisters_summary` with `update_canister_list = false`; `icq` never
asks Root to mutate its stored archive inventory. The sequential reads retain
typed relation gaps and do not claim one point-in-time snapshot.

Proposal and neuron collections:

```bash
icq sns proposal list 1 --limit 25
icq sns proposal info 1 387
icq sns proposal refresh 1
icq sns proposal cache status 1

icq sns neuron list 1 --sort api
icq sns neuron info 1 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
icq sns neuron refresh 1
icq sns neuron list 1 --sort stake --limit 500
icq sns neuron cache status 1

icq sns reward checkpoint 1 --json
icq sns reward checkpoint 23ten-uaaaa-aaaaq-aabia-cai --max-pages 10 --json
icq sns reward diff before-checkpoint.json after-checkpoint.json --json
```

SNS proposal list views can create a missing complete proposal cache and apply
supported filters and sorts locally. Proposal detail prefers a complete cache
that contains the row and otherwise performs a live lookup.

SNS neuron `--sort api` is a bounded live view. Whole-collection sorts require
a previously refreshed complete neuron snapshot so a normal view command
cannot hide an expensive crawl. Each row preserves the fixed-size native
Governance values already returned by `list_neurons`: stake and maturity,
creation and aging timestamps, source NNS neuron id, auto-stake setting, raw
dissolve state, voting-power percentage multiplier, vesting period, and fees.
JSON exposes all of those raw fields; compact text selects the operationally
useful subset. This adds no follow-up request or cache fanout. Neuron report
and cache schema 2 require an explicit refresh of older neuron snapshots.

`sns neuron info` is a separate live-only exact lookup. It accepts exactly one
32-byte neuron id as 64 lowercase hexadecimal characters and calls native
Governance `get_neuron` after targeted SNS discovery. The report preserves
every principal and raw permission code with its current native label, pending
maturity disbursement destinations including owner and subaccount, legacy and
topic followees, and the fixed-size state above. Unknown permission codes stay
visible and make the affected neuron-local maturity policy observation
unassessable. Including targeted discovery, the command makes exactly three
queries and never reads or writes the complete neuron cache.

`sns reward checkpoint` is a live API-exhausted observation, not a
point-in-time proof. It strictly walks native 100-row neuron pages and retains
each neuron's combined maturity, permissions, pending maturity disbursements,
and auto-stake state. Complete nervous-system parameters, latest reward event,
and running SNS version responses bracket the walk. Any bracket change,
duplicate or overlapping id, invalid cursor, missing exhaustion evidence, or
parameter-derived collection-bound violation fails without emitting a
checkpoint.

For `N` neuron pages the command makes `N + 8` client queries, including
targeted discovery. It never queries ballots, ledgers, transactions, or one
detail endpoint per neuron. `--max-pages` is an optional diagnostic ceiling;
reaching it before exhaustion is an error, not a partial result. Text output
is a bounded summary, while `--json` retains all raw neuron evidence for
explicit caller-managed persistence. The command does not read or write the
ordinary neuron cache and does not create a checkpoint file implicitly.

`sns reward diff` is local-only. It accepts two explicitly selected current-
schema checkpoint files and `--json`; it has no source endpoint and rejects an
explicit global `--network`. Both files are untrusted input. The library
recomputes raw maturity totals, row ids and order, permission findings,
parameter/event/version brackets, timestamps, counts, and stable target
principals before comparing them.

Rows join by the full 32-byte neuron id and always retain the signed raw
`maturity_delta_e8s_equivalent`. A positive allocation is reported only when
both policies are observed satisfied, the after reward event is the immediate
native successor by canonical end timestamp and round continuity, no neuron is
missing or unexplained, every delta is non-negative, and both aggregate and
per-neuron sums exactly equal the after event's
`distributed_e8s_equivalent`. Exact zero distribution produces
`no_allocation`; every failed invariant produces typed `invalid` evidence.
The command never infers a beneficiary, never writes a file, and explicitly
states that local checkpoint content is not authenticated.

## ICRC

Ledger-wide live reports:

```bash
icq icrc ledger capabilities mxzaz-hqaaa-aaaar-qaada-cai
icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc ledger index ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc ledger transactions ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc ledger block-types ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc ledger archives ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc ledger tip-certificate mxzaz-hqaaa-aaaar-qaada-cai
```

Transaction queries can follow ledger-supplied archive callbacks explicitly.
Tip-certificate reports authenticate certificate, delegation, canister
authority, freshness, certified data, and required tip leaves when the ledger
returns that evidence.

Account-scoped live reports:

```bash
icq icrc account balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa
icq icrc account allowance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa aaaaa-aa
icq icrc account transaction page mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa
```

Account-history collection separates live pages from complete local snapshots:

```bash
icq icrc account transaction refresh mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa
icq icrc account transaction list mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa \
  --sort oldest --limit 100
icq icrc account transaction cache status \
  mxzaz-hqaaa-aaaar-qaada-cai aaaaa-aa
```

`page` is a bounded live query. `refresh` resolves or accepts an explicit
index, verifies its ledger identity, exhausts pagination, and atomically
publishes a complete endpoint/ledger/account snapshot. `list` and
`cache status` are local-only.

The ICP ledger does not export ICRC-106 index discovery. Supply its official
index explicitly when querying ICP account history:

```bash
icq icrc account transaction page \
  ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa \
  --index-canister-id qhbym-qaaaa-aaaaa-aaafq-cai
```

## Cache root and recovery

The CLI selects one user-level cache root:

1. `ICQ_CACHE_ROOT` when set to an absolute path;
2. `$XDG_CACHE_HOME/ic-query`; or
3. `$HOME/.cache/ic-query`.

It never discovers or migrates repository-local `.icq` directories.

Complete snapshot refreshes use a lock. Paged proposal, neuron, and
account-history collections also use a separate attempt sidecar so a failed or
capped refresh can retain available progress while leaving the previous
complete snapshot unchanged. The exact-version joined topology cache uses its
lock and atomic replacement without an attempt sidecar. Stale or malformed
locks are reported but are not automatically deleted; remove a lock only after
confirming that no refresh is active.

Cache identity describes collected evidence, not its presentation. Sort,
limit, verbosity, and output format do not create alternate complete
snapshots. See [Cache Policy](design/cache-policy.md) for the full contract.
