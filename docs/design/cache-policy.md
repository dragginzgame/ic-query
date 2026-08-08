# Cache Policy

This note describes the shared cache behavior expected across `ic-query`.

## Goals

- CLI cache identity is user-level rather than repository-level. The CLI
  resolves one root from `ICQ_CACHE_ROOT`, `XDG_CACHE_HOME`, or `HOME`; library
  requests receive the actual root and never append a hidden `.icq` directory.
- Cache reads should be invisible when a complete current-schema snapshot
  exists.
- A missing cache should be created automatically only for read commands whose
  full refresh policy is fixed by the report layer.
- Recoverable invalid cache content should be replaced automatically only when
  the same operation already owns a bounded or explicitly selected live
  refresh policy.
- Commands whose complete snapshots can be expensive or require user-controlled
  page limits may require an explicit refresh before cache-backed reads.
- Live network calls must remain visible in output when a command refreshes or
  creates a cache.
- Cache keys describe collected data, not view options. Sorting, limits,
  lifecycle filtering, verbosity, and text formatting must not create separate
  complete snapshots.
- Failed refreshes should not replace a previously complete cache.
- Operators should be able to inspect every known complete cache and refresh
  lock, including age, size, and applicable stale policy, without making a
  network request.

## Managed Filesystem Authority

Managed loads, collection discovery, cache-status traversal, refresh locks,
attempt sidecars, and publication resolve from one opened capability root. A
managed path must remain beneath that root without parent traversal, and no
root, parent, or final component may be a symbolic link. Loads require regular
files. On Unix, managed directories deny group and other access, newly created
directories use mode `0700`, and managed cache and lock files use exactly
`0600`.

Confinement, nonregular-path, and unsafe-mode failures are filesystem authority
errors. Cache-only operations report them directly, and read-through policies
must not reinterpret them as invalid content that authorizes a live refresh.
Publication uses a same-directory exclusively created temporary file, syncs the
file, atomically renames it, and syncs its parent directory. Explicit
caller-selected exports are not managed cache files. There is no legacy reader,
permission repair, deletion, or migration for older permissive cache trees.

Managed pretty-JSON publication validates serialization before filesystem
mutation and then streams directly through the atomic temporary file, avoiding
a second complete encoded cache copy. An explicit caller-selected export may
retain one encoded string when the same bytes must also be published to cache.
Certified Registry archive objects and certified Subnet Catalog caches retain
their caller-selected read ceilings through the shared confined reader.
Refresh-lock reads are capped at 64 KiB and refresh-attempt sidecars at 1 MiB;
oversized metadata fails as invalid local evidence and never authorizes hidden
network work or automatic deletion.

## Shared Read-Through Flow

Cache-backed reads should follow this sequence:

1. Try to load the complete cache.
2. Classify a missing cache or a recoverable local content failure using the
   owning family's exact cache identity. Preserve filesystem and unrelated
   source failures.
3. Print the standard refresh announcement when the command owns visible
   progress, including the component name, cache path, and source endpoint.
4. Refresh or create the cache through the command-owned refresh path.
5. Load the cache again and build the report from the cached data.

Errors other than a missing cache are not refresh triggers by default. Parse,
schema, network-content, identity, and semantic failures may be recoverable
when the owner can reproduce the exact cache through an already-authorized
refresh. IO, permission, lock, and unrelated source failures remain errors.
Cache-only and status operations never opt into recovery.

`HostCacheError` is the canonical public owner of generic JSON cache read,
parse, serialization, schema, network-content, and shared file/lock operation
failures. Family host errors wrap it transparently and retain separate variants
only for owner-specific missing-cache guidance, identity, semantic validation,
or collection completeness. Owner-aware JSON loading preserves the specialized
missing error while mapping every other generic failure through this shared
taxonomy.

Use the owner-error-policy helpers when the operation already has:

- a cache loader
- a refresh implementation
- an exact expected cache path and typed recoverable-content errors

For small fixed-cost snapshots with an explicit age policy, use the distinct
refresh-if-stale flow. It refreshes a missing, owner-classified invalid, or
older complete snapshot. Read and permission failures must not be classified
as invalid content.

## Manual Refresh

Manual refresh commands always refresh explicitly and should report refresh
progress or status through their owning report modules. They do not need the
read-through helper because the user has already requested refresh behavior.

## Refresh Locks

Refresh locks record the network, target cache, owner process id, acquisition
time, and the stale threshold chosen by the refresh that created them. A
competing refresh honors that recorded threshold, so one caller cannot
reclassify another caller's active lock by supplying a shorter policy.

Refresh locks are never removed automatically. Parsed locks older than their
recorded stale threshold are reported explicitly as stale; malformed or
future-dated locks are reported as invalid. Commands show the lock path and
require the operator to remove it manually after verifying that no refresh is
still running. This avoids deleting a newly acquired lock during concurrent
stale-lock recovery.

## Cache Discovery

Cache status and cache list commands inspect local state only. Family-specific
list and status operations use their typed snapshot paths and validators. The
top-level `icq cache status` command performs a bounded cross-family inventory
of known complete-cache and refresh-lock filenames across every network
directory under the selected user-level root; it does not inspect
refresh-attempt sidecars as report rows, follow symlinks, refresh files, delete
files, or probe recorded process ids. It validates every traversed entry under
the capability root and rejects symbolic links or unsafe managed modes. The
scan bound applies to cache and lock candidates together.

Network-scoped caches use `<cache-root>/<domain>/<network>/...` as their common
top-level layout. NNS Registry leaf and Subnet caches therefore live below
`nns/<network>/`, alongside NNS Governance caches; SNS and ICRC caches use the
same domain-then-network ordering. Before 1.0, replaced path layouts are not
used by typed loaders or migrated automatically. The generic global inventory
may still expose an orphaned old file as unmanaged local evidence.

The global report separates generic header integrity from timestamp age.
`header_status` is `readable` or `invalid`; a readable header is not a claim
that the complete payload passed its owning family's semantic validator.
`age_status` is `fresh` or `stale` only for families with an explicit age
policy, `unmanaged` when a valid age has no threshold, and `unknown` when the
header or timestamp cannot supply an age. A malformed or future timestamp can
therefore remain a readable header with unknown age rather than collapsing two
different facts into one status.

The report sets `family_validation_performed` to false and derives registered
age thresholds and `recovery_policy` only from current canonical mainnet paths,
never from untrusted cache claims. `automatic` means an ordinary owner
read-through may replace recoverable invalid content, `explicit` requires a
selected refresh operation, `missing_only` means normal read-through creates
only absent content, and `unknown` identifies a file without a current
canonical owner. Large unmanaged proposal, neuron, and transaction histories
are inspected only through their leading header/completeness boundary, so
cross-family status does not load or scan complete row arrays. Small
age-managed files are fully JSON-parsed for syntax, but their family-specific
semantic validators remain authoritative.

Complete snapshot caches carry required logical identity fields and are
validated against the expected cache key on load. Identity-less snapshots are
unsupported and require an explicit refresh.
Complete snapshot loaders also reject unknown top-level fields and authority
claims that the owning source cannot make, including a true point-in-time
guarantee for paginated Governance or index histories. Current-shape loading
therefore cannot silently reinterpret a foreign or newer flattened snapshot.
Family-specific cache status and cache list commands render malformed,
unsupported, or identity-mismatched local snapshot files as invalid local
cache rows instead of silently ignoring them or making live calls. The global
inventory reports only failures visible at its generic bounded inspection
scope. Direct cache-only report reads also reject invalid snapshots; only the
owning read-through policies may replace them.

## Current Coverage

Bounded automatic read-through, including invalid-content recovery, is used by:

- subnet catalog list and information reports
- NNS node, node-provider, node-operator, and data-center list/information reports
- the joined deployed-SNS catalog

The shared NNS inventory boundary validates fixed canister identities, schema,
timestamps, endpoints, and declared row counts. Custom-source evidence is
rejected before publication when it does not match the exact refresh request.

The Subnet Catalog library exposes the underlying policy directly:
`CacheOnly`, `RefreshMissing`, `RefreshMissingOrInvalid`,
`RefreshMissingInvalidOrOlderThan`, and `ForceRefresh`. Every successful load
returns `CacheHit`, `RefreshedMissing`, `RefreshedInvalid`, `RefreshedStale`,
or `ForcedRefresh` with a private-field `ValidatedSubnetCatalog`. The caller
supplies the current time and any stale threshold; cache-only policy carries no
endpoint and cannot invoke a source. Ordinary CLI list/info behavior selects
missing-or-invalid repair and reports stale age without treating it as a
refresh instruction.

Every network-capable policy carries a `CatalogSourceSelection`, not an
implicit endpoint. It selects either one uncertified endpoint or a bounded
two-to-three-endpoint agreement collection. Async load/refresh entry points run
on the caller's runtime; synchronous names adapt the same implementation.
Load requests may require a minimum `CatalogAssurance`. Weaker cache evidence
fails as typed insufficient authority and is not silently classified as
missing, invalid, or stale. A refresh selection is checked against the same
minimum before collection, preventing a known-insufficient source from making
calls or replacing the cache. Successful outcomes can emit compact authority
evidence containing the exact Registry version, digest, assurance, endpoints,
and cache disposition.

The exact-version NNS Subnet topology and ICRC account-transaction libraries
apply the same recovery only through their explicit refresh-if-missing and
refresh-if-stale APIs. Their direct cache loaders remain local and strict.

SNS proposal list auto-cache creation remains missing-only. Numeric SNS ids
are resolved from cache headers, so an invalid header may not truthfully
identify which SNS should be recollected. Proposal and neuron histories can
also require complete Governance pagination and therefore retain explicit
invalid-cache recovery.

| Cache family | Missing-content policy | Invalid-content policy | Status recovery label |
| --- | --- | --- | --- |
| Subnet catalog | Automatic bounded refresh | Automatic bounded refresh | `automatic` |
| NNS node/provider/operator/data-center inventory | Automatic bounded refresh | Automatic bounded refresh | `automatic` |
| Joined deployed-SNS catalog | Automatic bounded refresh | Automatic bounded refresh | `automatic` |
| Exact-version NNS Subnet topology | Caller selects missing/stale read-through | Same selected read-through operation refreshes invalid content | `explicit` |
| ICRC account transactions | CLI is local-only; library caller may select read-through | Same selected library read-through operation refreshes invalid content | `explicit` |
| SNS proposals | Automatic only when the requested complete cache is unambiguously missing | Explicit refresh | `missing_only` |
| NNS proposals and NNS/SNS neurons | Explicit complete refresh or documented live fallback | Explicit refresh | `explicit` |

`sns list` uses a distinct one-hour refresh-if-stale policy for one bounded,
joined deployed-SNS catalog. The complete snapshot retains every SNS-W row,
Governance metadata result, and raw Swap lifecycle result. A fresh catalog
avoids all SNS-W, Governance, and Swap calls. Lifecycle selection is a view:
the default retains code `3` (`committed`, successfully launched), while
`--all` exposes every cached lifecycle and query-error row without changing
snapshot identity. Missing, stale, malformed, incompatible, identity-mismatched,
or semantically invalid content is visibly refreshed under one lock. The new
snapshot replaces the old path atomically only after validation, so a failed
refresh leaves the original invalid file in place. Cache-only and cache-status
operations still report the invalid evidence without a network call, and read
or permission failures remain errors. `sns refresh` forces replacement.
Targeted SNS commands retain targeted discovery and do not refresh or depend
on the all-SNS catalog.

The current registered age policies are:

| Cache | Stale after | Read behavior |
| --- | ---: | --- |
| Subnet catalog | 7 days | Refreshes missing or invalid content; reports stale age without replacing |
| Exact-version NNS Subnet topology | 24 hours | Explicit refresh-if-missing/stale APIs also replace invalid content |
| Joined deployed-SNS catalog | 1 hour | `sns list` refreshes missing, stale, or invalid content |

Other complete proposal, neuron, inventory, and transaction caches remain
`unmanaged` by age unless their owning operation explicitly defines a policy.

SNS proposal detail lookups opportunistically read an existing complete
proposal snapshot when the requested proposal row is present, then fall back to
the live detail API when the snapshot or row is missing. Cache parse, schema,
network, and IO errors remain visible instead of being hidden by fallback.

NNS neuron list and detail lookups follow the same cache-preferred,
live-fallback policy, but only an explicit `icq nns neuron refresh` writes the
complete snapshot. The public Governance index is ordered by neuron id and
supports bounded live pages; a full walk may be expensive and has no stable
point-in-time version. `icq nns neuron cache status` is local-only.

NNS Governance economics, cached metrics, latest reward-event, and
maturity-modulation reports are bounded live point-value queries. They do not
read or write the proposal or neuron complete-collection caches and do not
create another implicit cache or freshness policy.

Official Dashboard canister detail, filtered count, explicitly bounded page,
bounded metric time series, bounded daily statistics, boundary-node
data-center reports, and CloudEngine provider and explicit Type4 node reports
are live lookups. Count fetches no rows; page makes one request for at most 100
rows and never follows a cursor automatically. A
metric request selects one series family and is capped at 1,000 observations
per returned series. Daily statistics select one network-activity projection,
default to seven days, and are capped at one year and 366 rows. Boundary-node
data centers come from one non-paginated resource and do not trigger
per-location calls. These operations do not read or write a cache, because
their REST results are neither authoritative complete collections nor durable
point-in-time evidence. A future complete Dashboard collection or long-range
metric snapshot would require its own explicit operation, operational cap, and
timestamped identity, and must not reuse Registry or canister-authority caches.

CloudEngine provider list consumes one complete node-provider resource capped
at 1,000 rows and filters only after validation; exact provider info consumes
one bounded record. Their finite one-request shape does not justify an implicit
freshness policy, and their Dashboard identity must not reuse the separate
Registry Subnet Catalog or native CloudEngine control-plane cache boundaries.

CloudEngine node list consumes one explicitly filtered `Type4` node resource
capped at 10,000 rows; exact node info consumes one bounded record. Both are
live-only. Their reward/status/provider filter identity is distinct from the
default public-mainnet node-status snapshot, so they do not read or write its
60-second cache even though both scopes share raw row validation.

The certified API boundary-node report is also live-only, but for a different
reason: one bounded `read_state` request already returns a complete
authenticated subtree at one certificate time. It does not share the
Dashboard resource identity or node-status cache, and adding persistence would
require an explicit certificate-age policy that the current operation does not
need.

SNS neuron complete snapshots intentionally stay on explicit refresh before
cache-backed sorts. A full neuron refresh can require many governance pages and
the refresh command exposes `--page-size` and `--max-pages`; silently starting
that crawl from a normal sort command would hide important cost and completion
controls. Missing SNS neuron caches therefore remain typed user-facing errors
that point to `icq sns neuron refresh <id|root-principal>`.

ICRC account transaction lists also require an explicit complete refresh.
`icq icrc account transaction list` and `cache status` are local-only;
`transaction refresh` is the network-and-write operation. Library consumers
that explicitly want read-through behavior can choose the separate
refresh-if-missing or refresh-if-stale APIs. Endpoint, ledger, owner, and
subaccount form cache identity, while page size, cursor, list limit, and sort
do not. Failed refresh-attempt evidence retains the resolved index canister
when discovery or collection reached one, plus the latest page, row, and
cursor progress. It never publishes partial rows.
