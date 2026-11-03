# Command and evidence reference

[Quick start](../README.md) · [Command hierarchy](cli-usage.md) · [Rust APIs](rust-api-reference.md)

## Example command gallery

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

# Local analytics after explicit complete collection
icq nns proposal refresh
icq nns proposal activity --json
icq nns neuron refresh
icq nns neuron distribution --json

# Deployed SNS reports
icq sns list
icq sns list --all
icq sns refresh
icq sns canister list 1
icq sns metrics 1
icq sns upgrade 1
icq sns proposal list 1 --limit 25
icq sns neuron info 1 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f

# Generic ICRC reports
icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc account balance ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa
icq icrc analytics account count mxzaz-hqaaa-aaaar-qaada-cai
icq icrc analytics account list mxzaz-hqaaa-aaaar-qaada-cai --limit 25
icq icrc analytics account info mxzaz-hqaaa-aaaar-qaada-cai 222nw-nqiei-h4uy6-fqm54-d3slu-jveav-vqrn6-yojxi-4eug3-2ejie-vae
icq icrc analytics holder count mxzaz-hqaaa-aaaar-qaada-cai
icq icrc analytics holder list mxzaz-hqaaa-aaaar-qaada-cai --sort-by=-balance
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
subcommand, such as `icq sns neuron`. Every
`Commands` section is ordered alphabetically. The complete command map and
cache behavior are documented in
[CLI Usage](https://github.com/dragginzgame/ic-query/blob/main/docs/cli-usage.md).

### How to read the evidence

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-query/ic-query-answer-evidence.svg" alt="Three evidence categories: verified certificate, direct network response, and official dashboard observation">
</p>

An answer can accurately repeat its source without being a permanent fact
about the whole network. In particular, “official” does not automatically mean
cryptographically verified or captured at one exact moment.

Each report therefore records its **provenance**: where the data came from,
when it was retrieved, and what checks were possible. At a high level,
`ic-query` distinguishes three kinds of evidence:

| Kind | Plain-language meaning |
| --- | --- |
| Certificate-backed data | `ic-query` checked cryptographic proof that ties the response to the Internet Computer |
| Direct network response | A canister or other network service answered the query directly, but the response did not include the same certificate proof |
| Official dashboard observation | An official off-chain service reported what it observed at a particular time |

None of these labels makes an answer timeless. Network state may change after
a report is collected, and a source may describe only part of the network.
The detailed source families and their limits are compared below.

| Source | Evidence represented | Important limit |
| --- | --- | --- |
| Certified IC state tree | API boundary-node principals, domains, addresses, and certificate time authenticated against the built-in mainnet root key | Configuration does not prove operational health, reachability, HTTP-gateway membership, ownership, or physical location |
| NNS Registry | Authenticated certified latest-version and bounded contiguous delta-batch evidence, archive-bound exact-state catalog authority, plus exact-version joined Registry query evidence with explicit assurance | A single batch does not reconstruct Registry state; only a complete reauthenticated archive can promote a certified catalog, ordinary `get_value` reads remain uncertified, and endpoint agreement is not cryptographic certification |
| NNS/SNS canisters | Native adapters expose ordinary read-only replica query responses; the `canister` adapter exposes replicated inter-canister execution for the direct NNS Governance point reports plus bounded proposal/neuron pages and exact proposal/neuron detail | Replicated execution is not certificate evidence, and paginated or sequential calls may span state changes |
| ICRC ledger/index | Ledger queries, index analytics, and archive callbacks | Index histories expose API exhaustion, not a stable snapshot version |
| ICRC tip certificate | Certificate and hash-tree evidence verified by the host adapter | Verification applies only when the ledger returns the required evidence |
| Cycle Minting Canister | Application-level certificate and hash-tree witness verified against the CMC and returned rate | Cycles per ICP is derived from the certified rate and the documented one-trillion-cycles-per-XDR protocol constant |
| CloudEngine control plane | Ordinary public canister query responses from the fixed control-plane and resolved operator canister; list inventory separately comes from the versioned Registry Subnet Catalog | `certified: false`, `point_in_time_guaranteed: false` for control-plane observations; per-row calls do not become part of the Registry snapshot |
| Official IC Dashboard, including CloudEngine provider footprint and explicit Type4 nodes, observed default-scope node status, replica releases, node-provider rewards, and ICRC analytics | Timestamped off-chain REST analytics | `certified: false`, `point_in_time_guaranteed: false`; provider aggregates and Type4 node observations are separately timed, default node scope still excludes CloudEngine nodes, release records do not prove the binary currently running, reward offset pages may overlap, ICRC cursor pages are not complete snapshots, and an accepted ledger principal does not prove indexing coverage |

JSON reports keep raw identifiers, numeric fields, classifications, timestamps,
and explicit provenance. Text output may shorten or format values for people.
Report and persisted schemas are versioned independently. Every current schema
identifier remains `1` before 1.0; incompatible shapes replace that contract
in place without compatibility readers or automatic migrations.

See [IC Reporting Adapters](https://github.com/dragginzgame/ic-query/blob/main/docs/design/ic-reporting-adapters.md) for the
authority model and follow-up query rules.

### Command families

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
icq nns neuron cache|distribution|info|list|refresh
icq nns node info|list|refresh|status
icq nns node-operator info|list|refresh
icq nns node-provider info|list|refresh|status
icq nns node-provider reward history|info|list
icq nns proposal activity|cache|info|list|refresh
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

icq icrc account allowance|balance
icq icrc account transaction cache|list|page|refresh
icq icrc analytics account count <ledger-canister-id>
icq icrc analytics account info <ledger-canister-id> <account-id>
icq icrc analytics account list <ledger-canister-id>
icq icrc analytics holder count|list <ledger-canister-id>
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

### Collection and cache behavior

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
declared and streamed sizes through Host HTTP's fallible bounded reader before
JSON decoding. Query retains request, timeout and status policy. Indexed counts
request no account, holder, or transaction rows. A page returns at most 100 canister
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
receive the exact `CacheDisposition` used. Catalog loads bound cache files to
64 MiB before decoding and validate fixed
mainnet/Registry identity, raw Registry Subnet kinds, classification and
resolver policy identity, timestamps, canonical ordering, and the canonical
payload digest before returning a `ValidatedSubnetCatalog`. The digest detects
an inconsistent payload; it is not a signature and does not promote the
current `UncertifiedQuery` assurance. Bounded NNS Registry inventory
read-through operations retain their owner-selected invalid-content repair.
Successful `CacheDisposition` values describe how a catalog was supplied; they
do not describe a failed attempt. Detailed host loads instead return a
`SubnetCatalogLoadFailure` with the requested network, selected
`CatalogSourceSelection`, minimum assurance, exact `SubnetCatalogLoadStage`,
failure-side `SubnetCatalogFailureCacheDisposition`, optional pinned Registry
version, returned individual value version when available, source endpoint,
individual assurance, completed record evidence, typed subject, stable code/
category, and typed retryability. Once the live collector pins a Registry
version, every later Subnet-list, routing-key-family, routing-shard,
Subnet-record, decoding, validation, or aggregation failure retains that exact
requested version. A failure before version acquisition retains `None`, and
uncertain retryability is `Unknown(reason)` rather than a binary guess.
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
digest when applicable, exact Registry query-call counts, the selected routing
source, and per-value requested/returned versions, keys, schemas, subjects,
timestamps, endpoint, assurance, and inline/chunked representation. Agreement
does not become certified evidence and never falls back to one endpoint on a
mismatch.

Custom sources and downstream fixtures can construct that evidence without
hand-formatting Registry keys. The portable API exposes the canonical key
constants, typed Subnet-list, legacy-routing, routing-shard, and Subnet-record
subject constructors, and
`SubnetCatalogRegistryRecordEvidence::uncertified_query` for one ordinary
pinned value response.

Mainnet routing authority follows the current Registry helper contract. At the
pinned Registry version, the collector reconstructs the complete present-key
set under `canister_ranges_*`, fetches and decodes every shard at that same
version, and flattens the shards into one strictly validated routing table. If
the modern family is empty, live collection fails closed; the retired
monolithic `routing_table` is never queried or considered. Missing, empty,
malformed, overlapping, contradictory, shard-boundary-violating, or unknown-
Subnet shard evidence likewise fails closed, so a stale legacy value cannot
resurrect a deleted Subnet or mask lost modern authority. Size-bounded
`get_changes_since` pages advance by their highest returned mutation version;
the response's latest-version field is not treated as a page watermark.

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

SNS collection cache lists and numeric-id lookups bound local discovery to
1,024 candidates and 16,384 directory entries per scan, with a shared 1 GiB
read allowance across snapshots and attempt evidence. Exceeding a bound returns
an error rather than an incomplete list or lookup result. Root-principal
operations use their direct paths and existing per-file limits.
Numeric reads also verify the loaded snapshot's id after discovery, rejecting
an atomic replacement that changes it instead of returning another SNS.

SNS neuron/proposal cache lists and direct cache status both return errors for
malformed or invalid refresh-attempt files. A missing attempt is valid absence.
Cache lists still show invalid snapshots as invalid rows when their attempt
evidence is absent or valid; they never refresh or repair files implicitly.

The exact-version joined topology cache uses one refresh lock and atomic
replacement without a separate attempt sidecar. Collection limits and cursors
are operation controls; sorts, view limits, verbosity, and output format do
not change snapshot identity.

The CLI uses one user-level cache root in every working directory. It selects:

1. `ICQ_CACHE_ROOT`, which must be an absolute path;
2. `$XDG_CACHE_HOME/ic-query` when `XDG_CACHE_HOME` is nonempty and absolute; or
3. `$HOME/.cache/ic-query`, with an absolute `HOME`.

Empty values are ignored. A relative `ICQ_CACHE_ROOT` is an error; a relative
`XDG_CACHE_HOME` is ignored in favor of `HOME`.
It does not inspect project files or read and migrate former project-local
`.icq` directories. Cache semantics and recovery rules are defined in
[Cache Policy](https://github.com/dragginzgame/ic-query/blob/main/docs/design/cache-policy.md).
Network-scoped cache paths consistently begin with
`<cache-root>/<domain>/<network>/...`.

Managed cache loads, discovery, refresh locks, and publication are confined to
that root. On Unix, symbolic links, path escapes, nonregular managed files,
and group/world-writable directories or files are rejected. Readable
directories such as `0755` and files such as `0644` are supported without
permission repair. New managed directories use mode `0700`; new cache and lock
files use mode `0600`. These authority failures are not treated as invalid JSON
that a read-through call may silently replace. Explicit caller-selected output
files are outside this cache policy, but refresh exports must not alias the
managed snapshot or its refresh lock, including during a dry run.

Use `icq cache status` to inspect known complete caches across that root,
including generic header integrity, separate fresh/stale/unmanaged/unknown age,
sizes, stale thresholds, and automatic/explicit/missing-only invalid-content
recovery policy. The bounded inventory explicitly reports that it did not
perform family-specific semantic validation. It also reports active, available,
and invalid refresh coordination files without live calls, process probes, full history
scans, or cache mutation. Cache content inspection shares a 1 GiB byte budget
across the inventory; exhaustion returns an error rather than a partial success.

Subnet Catalog history transcripts and their writer lock appear as
`nns/registry-history`. Their unmanaged age is separate from catalog freshness;
full transcript validation occurs only during authorized live acquisition.
