# CLI Usage

`icq` exposes read-only reports grouped by the authority or protocol family
that supplies them. This guide describes the stable command hierarchy and
collection behavior. Run a command with `help` for its complete current option
reference.

## Common behavior

```bash
icq help
icq nns help
icq nns topology summary help
```

Report commands use human-facing text by default and accept
`--format json` for raw, script-friendly output. JSON reports retain stable
identifiers, native numeric fields, source endpoints, collection timestamps,
and authority guarantees.

All current report schemas use `schema_version: 1`. Before 1.0, incompatible
shape changes replace the previous contract without aliases or legacy readers.

## Target identity

NNS and SNS commands accept the global network identity:

```bash
icq --network ic nns registry version
icq --network ic sns list
```

The built-in sources and caches currently support only mainnet, named `ic`.
A different network is rejected before a live adapter is constructed.

Official Dashboard and ICRC commands identify their target using an entity
principal plus `--source-endpoint`. They reject the top-level `--network`
option instead of silently ignoring it.

## Official IC Dashboard

```bash
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --format json
icq ic canister count --has-name true
icq ic canister page --query ledger --limit 25 --format json
icq ic canister page --after ryjl3-tyaaa-aaaaa-aaaba-cai --limit 25
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai \
  --source-endpoint https://ic-api.internetcomputer.org/api/v3
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
`--before` to another command. There is no automatic enumeration and these
commands never read or write a cache. The official Dashboard is an off-chain
analytics authority, so every report states
`certified: false` and `point_in_time_guaranteed: false`. It does not inherit a
Registry version or prove current controller/module state.

See
[IC Dashboard Canister Reporting](design/ic-dashboard-canister-reporting.md)
for the exact report and validation contract.

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
icq nns governance metrics --format json
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

## SNS

Resolve a deployed SNS by list id or Root principal:

```bash
icq sns list
icq sns info 1
icq sns token 1
icq sns params 1
```

Inspect Root membership and read-only operational health:

```bash
icq sns canister list 1
icq sns canister list 23ten-uaaaa-aaaaq-aabia-cai --format json
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
icq sns neuron refresh 1
icq sns neuron list 1 --sort stake --limit 500
icq sns neuron cache status 1
```

SNS proposal list views can create a missing complete proposal cache and apply
supported filters and sorts locally. Proposal detail prefers a complete cache
that contains the row and otherwise performs a live lookup.

SNS neuron `--sort api` is a bounded live view. Whole-collection sorts require
a previously refreshed complete neuron snapshot so a normal view command
cannot hide an expensive crawl.

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
