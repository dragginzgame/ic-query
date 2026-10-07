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

## Bounded Content And Discovery

Shared JSON cache owners admit at most 64 MiB for inventories, observed node
status, joined SNS discovery, and exact Subnet topology, or 512 MiB for complete
NNS/SNS proposal and neuron histories and ICRC account transactions. The
confined reader checks metadata before allocating and stops streamed growth at
the ceiling plus one byte. Oversize content returns `HostCacheError::CacheTooLarge`.
Invalid UTF-8 is a JSON content failure. Both follow the owner's existing
invalid-content recovery policy; genuine IO and confinement failures remain
errors. Failed repair preserves the previous file. Subnet Catalog uses the
same 64 MiB ceiling for reads and publication, retaining its typed catalog
read failures.

The shared UTC timestamp parser accepts only the canonical second-precision
text emitted by the timestamp formatter, such as `2026-06-04T00:00:00Z`.
Noncanonical text follows each owner's existing invalid-content or freshness
error path; reads do not normalize or migrate stored values.

ICRC account-history collection timestamps use the caller's supplied start
clock plus monotonic elapsed collection time for completion. Snapshot reads
require canonical timestamps with completion at or after start. Reversed
intervals are invalid content under the existing explicit read-through policy;
cache-only reads reject them. A refresh with an unrepresentable interval fails
before replacing the previous snapshot. Refresh-attempt update timestamps
remain local wall-clock observations, separate from this collection interval.

Account-history cursors share unsigned ASCII-decimal validation. Request
normalization strips leading zeroes directly from validated text; native index
queries convert to Candid `Nat` only for wire arguments. Attempt reads validate
cursor text without allocating and discarding a normalized value. Arbitrary-size
ids and leading-zero request/attempt text remain supported, while stored rows
and returned page cursors retain their canonical newest-first requirements.

ICRC publication validates caller-supplied collection rows, index identity,
page evidence, the requested page cap, and final cursor before building the
snapshot. It constructs and validates completeness once through the shared
validator, then moves that evidence into the snapshot. Request identity and
newest/oldest ids are constructed from the validated inputs without a second
row-validation pass. Publication and stored-snapshot
loading share the canonical ordered timestamp check; disk reads retain full
snapshot identity, completeness, index, and row validation.

The shared completeness validator rejects row counts exceeding page size times
page count. It widens before multiplication and permits exact capacity and
empty exhausted collections. SNS discovery records one complete inventory
response with its observed width as `page_size` (at least one for an empty
inventory), replacing placeholder page evidence. Contradictory persisted
counts are invalid content: cache-only reads remain strict, while the owning
read-through policies visibly recollect them. No fallback reader or migration
is added, and schemas remain `1`.

Strict shared loads validate top-level duplicates, supported fields, schema,
and network in one header pass, then deserialize the typed report. An isolated
local harness over the actual old/new loaders, with shared regular-file IO
stubs, measured five warmed samples per case: median time fell from 0.299 s to
0.221 s for 13.7 MB and from 1.827 s to 1.445 s for 68.5 MB. These compare JSON
loading work, not complete native cache or network performance.

Cache-status discovery queues paths rather than live directory handles. It
caps selected files at the requested scan limit, all visited entries at 16
times that limit, and directory depth at 32. Reaching any bound reports
`truncated: true`; paths reopened during traversal retain capability-rooted
permission and link checks.

Successful refresh-lock removal ends the guard's ownership before directory
sync. A sync error is returned without attempting to unlink a subsequent
owner's replacement lock.

## Managed Filesystem Authority

Managed loads, collection discovery, cache-status traversal, refresh locks,
attempt sidecars, and publication resolve from one opened capability root. A
managed path must remain beneath that root without parent traversal, and no
root, parent, or final component may be a symbolic link. Loads require regular
files. On Unix, managed directories and files deny group and other write access.
Readable directories such as `0755` and files such as `0644` are supported:
reports contain public metadata, and confinement protects the integrity of that
evidence rather than requiring confidentiality. Newly created directories use
`0700`, and newly published cache and lock files use `0600`. Existing permissions
are never automatically changed.

Confinement, nonregular-path, and unsafe-mode failures are filesystem authority
errors. Cache-only operations report them directly, and read-through policies
must not reinterpret them as invalid content that authorizes a live refresh.
Publication uses a same-directory exclusively created temporary file, syncs the
file, atomically renames it, and syncs its parent directory. Explicit
caller-selected exports are not managed cache files. Refresh exports must not
alias the operation's managed cache or refresh lock, even during a dry run.
Subnet Catalog exports also protect the managed Registry history transcript
and its writer lock, including a caller-owned source's configured history paths.
Compare resolved paths before creating managed directories or acquiring the lock,
including missing targets and symlink aliases, then check again when writing.
On Unix, also compare file identities to reject hard links; open exports without
truncation and check their opened identity before clearing their contents.
Group/world-writable paths remain errors; there is no permission repair,
deletion, or migration for them.

Atomic replacement is not rollback. A parent-directory sync error after rename
is returned as a durability failure even though the new snapshot is already
visible. Source, validation, and temporary-write failures before replacement
leave the previous snapshot intact.

Managed pretty-JSON publication validates serialization before filesystem
mutation and then streams directly through the atomic temporary file, avoiding
a second complete encoded cache copy. Both serialization passes enforce the
same byte ceiling as the owner's reader; an oversized refresh returns
`CacheFileError::WriteLimitExceeded` before replacement. The temporary-file
pass remains bounded if serialization changes between passes. An explicit
caller-selected export may retain one encoded string when the same bytes must
also be published to cache.
The cache-bearing native host features use `ic-host-artifacts`' `BoundedWriter` for
byte counting and output budgets, and `read_reader` for bounded, fallibly allocated
byte collection from already-confined handles. Its `MatchingWriter` compares
canonical JSON without retaining another encoded copy; the JSON producer must
succeed before a complete match can be accepted. Metadata admission, observed
overflow lengths, aggregate charging, encoding, typed cache errors, confinement and
publication stay with IC Query. Canonical hashes stream into SHA-256's existing
writer. Its default features stay disabled: cache writes do not enable archive,
gzip, Wasm inspection, process execution or ordinary-path filesystem helpers.
Pure-library, canister and host features without caches do not enable any IC Host
Tooling crate. The development-only Governance artifact helper selects Wasm
inspection and artifact file reads separately from production library dependencies.
Stream allocation failures return a read error with `OutOfMemory` rather than
panicking while preallocating a metadata-sized buffer.
Certified Registry archive objects and certified Subnet Catalog caches retain
their caller-selected read ceilings through the shared confined reader.
Ordinary Subnet Catalog refreshes stream through the bounded JSON writer;
only explicit exports retain the full pretty-JSON string. Dry-run validation
without an export uses a serialization sink. Exports that also publish a
snapshot must fit the reader's ceiling before either file is replaced.
Refresh-lock reads and writes are capped at 64 KiB and refresh-attempt sidecars
at 1 MiB;
oversized metadata fails as invalid local evidence and never authorizes hidden
network work or automatic deletion.

Live Subnet Catalog convenience calls also retain bounded Registry history
transcripts under the same root. These schema-1 files are an endpoint-isolated
query optimization, not complete catalog authority or an age exemption.
Caller-owned sources opt in explicitly with `with_history_cache(root)`.
Invalid transcripts can be replaced only through already-authorized live
collection; confinement and IO failures remain errors. Cache-only and cache-hit
paths never inspect or refresh them. See the
[transcript contract](subnet-catalog-acquisition-performance.md#cross-process-transcript-contract)
for bounds, local-owner trust, atomic publication, and concurrent-writer policy.

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

Use the shared read-through runner with an owner-specific error classifier when
the operation already has:

- a cache loader
- a refresh implementation
- an exact expected cache path and typed recoverable-content errors

The same runner accepts the owner's freshness predicate. Missing-only and
missing-or-invalid reads disable age-triggered refresh; snapshots with an
explicit age policy select their stale predicate. Error classifiers retain
each family's recovery rules. Read and permission failures must not be
classified as invalid content.

## Manual Refresh

Manual refresh commands always refresh explicitly and should report refresh
progress or status through their owning report modules. They do not need the
read-through helper because the user has already requested refresh behavior.

Each complete SNS proposal or neuron collector owns its paged state directly.
It validates source pages before ingesting rows and maps its family-specific
cursor. Proposal pages are unique within each page and strictly below the
previous minimum id, so they cannot overlap. Continuation uses the lowest
returned id without changing source row order. Neuron pages must have ascending
ids and any supplied cursor must equal the final row. Inclusive boundary overlap
remains supported; the neuron adapter removes that one repeated row and retains
its first observation. Older rows are invalid, so no other overlap is possible;
full pages require an advancing cursor, while short terminal pages may omit it.

Strict neuron-page ordering also establishes within-page uniqueness, so page
validation does not build a second id set. Bounded results and stored snapshots
permit unordered rows and retain set-based uniqueness validation. Proposal and
neuron source results share the same capability-aware requested-row limit check.

Shared paging state owns counters, admitted rows, and the next cursor, without
a collection-wide duplicate registry. Exhaustion uses the original API page
length, including any boundary row, rather than the number of newly admitted rows.
Invalid pages fail before ingestion and cannot replace a complete snapshot.
The shared refresh runner detects page limits and stalls and drives
progress events and running-attempt updates before completion.

## Refresh Locks

Refresh locks record the network, target cache, owner process id, acquisition
time, and the stale threshold chosen by the refresh that created them. A
competing refresh honors that recorded threshold, so one caller cannot
reclassify another caller's active lock by supplying a shorter policy.

The refresh guard owns its lock immediately after exclusive creation. Writing
and syncing the lock file precede syncing the parent directory; failure at
either stage drops the guard and attempts to remove the newly created lock.
This does not authorize removal of another refresh's lock.

Existing refresh locks are never removed automatically. Parsed locks older
than their recorded stale threshold are reported explicitly as stale; malformed or
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

SNS and NNS proposal cache-list summaries read refresh-attempt sidecars once
on a best-effort basis. Their exact cache status reads the selected sidecar
once through the strict reader and reuses that observation in both the snapshot
summary and the top-level attempt field. Invalid attempt content remains a status error;
snapshot and attempt files are still separate observations, not an atomic pair.
Status and failed-refresh progress recovery use one validated SNS attempt
reader. Failure recording treats unreadable or invalid prior attempt evidence
as absent and keeps the original refresh error; exact status remains strict.

Complete paged refreshes share the lock and attempt lifecycle helpers.
Failure-sidecar writes are best effort and preserve the original refresh error. Snapshot
publication precedes attempt finalization; a failed finalization is exposed in
the successful refresh report as `attempt_finalization_error`, without marking
the published snapshot as failed. NNS and SNS recover the latest valid page
progress from their running sidecar, while ICRC collection errors carry their
progress and resolved index identity directly. These are distinct evidence
contracts under the same lifecycle, not competing recovery coordinators.

Resolved live SNS identity, including its original SNS-W inventory position,
belongs to `MainnetSns`. Live reports, complete snapshots, and refresh-attempt
sidecars project that same id. Numeric and Root-principal lookup preserve the
full inventory position even though metadata enrichment joins only one target.
The shared refresh context borrows the operation's snapshot, lock, and attempt
paths; lock acquisition and publication use those same paths.

Numeric SNS cache reads bind the selected snapshot's loaded id to the requested
id after header discovery. If atomic publication changes that id between reads,
the operation returns `SnsHostError::CacheIdentityMismatch` instead of reporting
another SNS. Replacement with the same id remains readable. Header discovery
does not provide an atomic view of all collection files.

SNS collection discovery admits at most 1,024 candidate files and 16,384
network-directory entries per scan, including entries without a matching
collection. Cache lists and numeric-id lookups share a 1 GiB read allowance
across candidate headers, selected snapshot loads, and attempt sidecars. Numeric
status retains that allowance when falling back to attempt-only discovery.
Bytes read from malformed files count too; metadata admission occurs before
allocation. Streamed growth stops at the remaining allowance plus one detection
byte, and exceeding the allowance ends the operation.
Partial IO failures conservatively consume their admitted read ceiling.

Exceeding a discovery or aggregate byte bound returns
`CacheFileError::ScanLimitExceeded` through the SNS cache-operation error.
It never produces a partial list, a supposedly unique id, a missing-cache
result, or a refresh trigger. Invalid files within the allowance retain the
existing strict-read, invalid-summary, or best-effort sidecar policy. Direct
root-principal operations do not scan unrelated entities and retain the
512 MiB snapshot and 1 MiB attempt limits. These bounds add no live calls or
cache mutation; downstream exhaustive matches on `CacheFileError` must handle
the new variant. Schemas and serialized shapes remain unchanged.

Global status applies a 64 MiB byte ceiling to full inspection of age-managed
files, using the shared bounded reader to reject oversized metadata and growth
after opening. Unmanaged inspection consumes at most 64 MiB before reaching
the payload boundary; the unread payload may be larger. Exhausting that budget
without a readable header produces an invalid inspection row. These limits do
not authorize refresh or deletion, and rows retain the observed file size.

Complete-snapshot refreshes inspect whether they replace an existing snapshot
only after acquiring its refresh lock. The replacement flag describes state
seen by the publishing owner, rather than an earlier observation outside the
lock.

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
- observed Dashboard node-status snapshots shared by NNS node/Subnet/provider
  status views

Observed Dashboard node-status refresh moves validated rows and provenance into
the complete snapshot envelope, retaining the computed count summary for its
refresh report. Publication does not clone the row collection. Cache reads and
public snapshot projections retain their independent validation boundaries.

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

Ordinary Subnet Catalog cache reads have a fixed 64 MiB byte ceiling. The
confined reader checks the opened regular file's length before allocation and
reads at most the ceiling plus one byte to detect growth before UTF-8 or JSON
decoding. Oversized content returns `CachedCatalogTooLarge` with its path,
observed length, and ceiling; detailed failures use `cached_catalog_too_large`,
the `validation` category, and `CacheRejection` / `CacheRejected` provenance.
`CacheOnly` and `RefreshMissing` return that rejection without source calls.
`RefreshMissingOrInvalid` and `RefreshMissingInvalidOrOlderThan` may replace it
through their explicit source selection and report `RefreshedInvalid`.
`ForceRefresh` bypasses the cache read. A failed repair preserves the prior
file and releases the refresh lock. Confinement, permissions, filesystem IO,
and UTF-8 failures remain strict errors rather than repair authority.

Every network-capable policy carries a `CatalogSourceSelection`, not an
implicit endpoint. It selects either one uncertified endpoint or a bounded
two-to-three-endpoint agreement collection. Async load/refresh entry points run
on the caller's runtime; synchronous names adapt the same implementation.
Load requests may require a minimum `CatalogAssurance`. Weaker cache evidence
fails as typed insufficient authority and is not silently classified as
missing, invalid, or stale. A refresh selection is checked against the same
minimum before collection, preventing a known-insufficient source from making
calls or replacing the cache. Successful outcomes expose `snapshot_authority()`
with the exact Registry version, digest, assurance, and canonical endpoints.
Cache path and disposition remain separate acquisition diagnostics; they do
not enter stable snapshot authority identity.

`CacheDisposition` is success-only evidence: it says whether the returned
catalog was a hit, refreshed missing/invalid/stale content, or came from a
forced refresh. It must not be reused to describe a failure. Detailed Subnet
Catalog loads use `SubnetCatalogLoadStage` plus
`SubnetCatalogFailureCacheDisposition` for failure provenance. Together they
distinguish cache-only loading, forced cache bypass, missing and rejected
content, refresh preflight, refresh failure with its missing/rejected/stale/
forced trigger, and a post-refresh cache-load failure. The detailed failure
separately retains the optional pinned Registry version, typed subject, stable
error classification, retryability, and original host error.

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
| Observed Dashboard node status | Automatic bounded refresh | Automatic bounded refresh | `automatic` |
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
The shared catalog fetcher validates inventory provenance and canister identities,
then admits exact-target metadata and lifecycle results before joining them.
Joining moves admitted fields, supplies a nonempty fallback name, and assigns
ids in SNS-W order; live reporting and publication do not repeat that validation.
Stored catalogs validate borrowed rows, including required lifecycle evidence,
in one pass at the disk boundary; cache-backed list views consume the loaded
rows without reconstructing a catalog for validation or cloning it for filtering
and sorting. View projection leaves the complete snapshot unchanged.
Targeted SNS commands retain targeted discovery and do not refresh or depend
on the all-SNS catalog.

The current registered age policies are:

| Cache | Stale after | Read behavior |
| --- | ---: | --- |
| Subnet catalog | 7 days | Refreshes missing or invalid content; reports stale age without replacing |
| Exact-version NNS Subnet topology | 24 hours | Explicit refresh-if-missing/stale APIs also replace invalid content |
| Joined deployed-SNS catalog | 1 hour | `sns list` refreshes missing, stale, or invalid content |
| Observed Dashboard node status | 60 seconds | NNS node/Subnet/provider status reads refresh missing, stale, or invalid content |

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

`icq nns proposal activity` and `icq nns neuron distribution` require the
complete local snapshot and never fall back to a live query or write cache
data. Starting with 0.46.0, NNS proposal and neuron snapshots retain their
actual final `collection_state`. Readers validate agreement with the cache
envelope before using that evidence in the existing pure analytics. The
schema-1 shape is a hard cut: explicitly refresh existing snapshots; no
older reader or automatic migration is supported.

NNS Governance economics, cached metrics, latest reward-event, and
maturity-modulation reports are bounded live point-value queries. They do not
read or write the proposal or neuron complete-collection caches and do not
create another implicit cache or freshness policy.

Official Dashboard canister detail, filtered count, explicitly bounded page,
bounded metric time series, bounded daily statistics, boundary-node
data-center reports, node-provider reward reports, and CloudEngine provider
and explicit Type4 node reports are live lookups. Count fetches no rows; page
makes one request for at most 100
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

Node-provider reward detail consumes one exact Dashboard record, list consumes
one page of at most 100 records, and aggregate history consumes one bounded
start/end/step response with at most 1,000 requested observations. Offset pages
can overlap even under one maximum reward index, so list is not a complete
collection protocol and must not acquire a cache identity or automatic paging
policy. These reports do not reuse the NNS node-provider inventory or observed
node-status caches.

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

Single-page ICRC account reports and native complete collections share page
validation. Rows are canonical newest-first ids below the exclusive requested
cursor and at or above any supplied oldest id; the next cursor equals the final
row. A rejected page leaves accepted rows, balance, page counts, and cursor
unchanged. Ordered pages and exclusive boundaries establish global uniqueness
and order during collection, so finalization does not sort or deduplicate again.
Disk loads and caller-supplied complete collections retain their own boundary
validation. Sparse ids, arbitrary-size naturals, and empty terminal pages remain
supported; exhaustion still does not establish a point-in-time snapshot.
