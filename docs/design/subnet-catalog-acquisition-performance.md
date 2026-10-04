# Subnet Catalog acquisition performance

The collection and transcript contracts below describe the current code.
Dated measurements and downstream reviews preserve evidence from the reviewed
checkout; they do not establish a downstream repository's current adoption.
Cross-process history reuse and export-alias protection are implemented here.

## Collection policy

Agreement collection polls at most two endpoint futures at once. Each endpoint
still independently pins its latest Registry version, reconstructs routing-key
membership, and reads every catalog record at that pin. Validation and error
selection retain canonical endpoint order. Promotion still requires exact
Registry-version and canonical-content equality; neither a mismatch nor an
endpoint failure permits weaker evidence. The stream owns its futures, so
cancellation drops both active collections before the refresh lock is released.
A later endpoint's failure can wait behind an earlier endpoint's pending read;
the caller's acquisition deadline remains the overall bound.

`LiveSubnetCatalogSource` is a reusable, cloneable source for the existing
`*_with_source` APIs. Its clones share memory history checkpoints and an optional
structured progress callback. Keep one instance across retry and refresh
attempts, or configure `with_history_cache(cache_root)` to reuse history across
fresh sources and processes. Standard live catalog load/refresh convenience
APIs configure disk history under the request's cache root. Cache hits and
cache-only reads do not inspect or write history. Dry-run refresh convenience
calls use memory-only history; caller-supplied sources retain their explicit
configuration.

In-memory checkpoint handles and normalized page payloads are shared between
collections. Retained-byte accounting advances with each admitted page instead
of rescanning the transcript. Catalog annotation validation copies only Subnet
rows, and catalog and agreement digests stream canonical JSON into SHA-256
without allocating a complete encoded copy. Persisted transcript and digest
contracts remain unchanged.

## History reuse and authority

Checkpoints are keyed by exact endpoint string and key-family prefix, with at
most eight retained entries per source. Each contains the membership state
(including tombstones), highest completely validated version, and cumulative
resource counters. There is no global cache. Disk reuse is confined to the
caller-selected root and never creates a second catalog authority.

A page is retained only after the existing continuity, key/value ceiling,
mutation, and deletion validation succeeds for the entire page. Mutations after
the pin are ignored; the checkpoint watermark is capped at the pin, never the
response's latest version. A malformed page leaves the prior checkpoint intact.
Cancellation or later value acquisition failure may retain already validated
prefixes, but never publishes a partial catalog.

For a later pin, discovery starts at that endpoint's checkpoint and applies
subsequent changes. At the same pin, discovery needs no history requests. An
older pin starts at zero; a newer checkpoint cannot answer an older request.
Concurrent users never replace a newer checkpoint with an older one. Exceeding
the eight-entry retention cap disables retention for additional identities;
collection still independently scans and validates their history.

This relies on Registry versions being immutable, as do exact-version value
reads. Checkpoints retain ordinary query evidence from the same endpoint;
they are neither certified nor independent endpoint evidence. No endpoint can
seed another endpoint's discovery. Every Subnet list, routing shard, and Subnet
record is freshly read at the new pin, and existing agreement, assurance-floor,
per-record provenance, and catalog validation remain mandatory. No key list is
inferred from Subnet membership or hard-coded shard introduction versions.

Without a configured or valid disk checkpoint, collection scans history from
zero. Cumulative resource ceilings remain conservative, including values beyond
the pin that were inspected in a boundary page.

### Cross-process transcript contract

The current schema-1 file is `nns/ic/subnet-catalog/history.json`, beside the
catalog and guarded by a separate `history.lock`. It retains at most eight
exact endpoint/prefix identities, each also bound to canonical network `ic`
and the mainnet Registry canister. The complete JSON file is capped at 64 MiB;
each transcript has at most 4,096 pages, and normalized page bytes retain the
agent's 8 MiB response ceiling. Existing 100,000-delta-key, one-million-value,
and 1,024-byte-key ceilings still apply cumulatively after restoration.

The file stores normalized protobuf delta pages as lowercase hexadecimal,
including all keys, mutation versions, presence/deletion markers, observed
latest versions, and each page's validated watermark. Inline payloads, chunk
contents/references, and mutation timestamps are unnecessary for key discovery
and are discarded. Restoration replays all retained pages through the same
continuity, mutation, deletion, and resource validator. It reconstructs both
membership and tombstones and recomputes cumulative counters. Boundary-page
mutations after that page's watermark remain ignored even when restoring for a
later pin. No serialized membership map is accepted as sufficient evidence.

This is **trusted local-owner query evidence**. The root and files use the
existing no-symlink confinement and reject group or other write access.
Readable `0755` directories and `0644` files are accepted without permission
repair; newly created directories and files use `0700` and `0600`,
respectively. A SHA-256 checksum binds the current schema, network, publication
timestamp, identities, and transcripts against accidental corruption; it does
not authenticate Registry responses or prove completeness against an owner who
rewrites the transcript and checksum. Same-account malicious modification is
outside this trust contract, as it is for ordinary catalog caches.
Certification would require separately retained authenticated evidence.
Restored prefixes never raise assurance, exempt freshness, seed another
endpoint, or replace fresh pinned Subnet-list, routing-shard, and Subnet-record
reads and endpoint agreement.

Malformed, wrong-identity, unsupported-schema, oversized, checksum-invalid,
or semantically invalid local content emits `HistoryCache::Rejected` progress
and starts from zero only within the already-authorized live operation.
Permission, confinement, and other IO failures remain typed cache errors and
do not authorize a fallback. Cache-only policy never uses history to start
network work. Invalid bytes remain untouched until a complete live page is
validated and atomically published; a failed collection preserves them.

Memory checkpoints advance after every fully validated page. Disk publication
occurs after the first page, every eight collected pages, and history completion,
before another await. This avoids rewriting the growing transcript for every
page. Ordinary interruption can leave up to seven completed pages to repeat in
a new process; the durable prefix remains complete. Cancellation or later
record failure never publishes a partial catalog. A short filesystem lock
serializes reread/merge and atomic replacement; writers preserve other identities
and never replace a newer valid prefix with an older one. A busy writer or retention ceiling
emits `Skipped` and leaves live collection intact. Stale or invalid locks
remain errors and require operator inspection and manual cleanup. Skipped
publication can leave an older prefix than the ordinary seven-page window.
There are no detached writers or automatic lock deletion. Progress callbacks
run after both history mutexes and filesystem locks have been released.

`icq cache status` lists the transcript and its lock as `nns/registry-history`.
It reads the leading metadata without scanning transcripts and reports no age
expiry policy; history publication time is not catalog freshness or authority.
Full transcript validation belongs to live acquisition, not generic inventory.

## Progress and retries

`LiveSubnetCatalogSource::with_progress` accepts a synchronous `Send + Sync`
callback receiving `SubnetCatalogProgress`:

- exact endpoint and endpoint-local query-attempt count;
- endpoint start and pinned Registry version;
- history watermark, target version, and whether the prefix was reused;
- disk checkpoint path, read/write disposition, watermark, and rejection or
  skipped-publication reason;
- record key, requested version, and read start/completion;
- retry method, next attempt, and delay in milliseconds.

A completed record event means the value transport/decode completed, not that
the final catalog has passed validation. Chunk calls contribute to the query
count. Events are transient; they do not enter catalog JSON, snapshot authority,
or failure provenance. Callbacks run without the history mutex held and should
return promptly. There is no library output sink, background task, heartbeat,
or acquisition deadline. Cache hits do not invoke the source or callback.

Catalog queries retry connection errors, transport timeouts, and HTTP 502/504
at most twice, after 250 ms and 500 ms. Requests retain the same endpoint,
method, key, and pin/cursor/hash. The final error follows the existing typed
failure path, including requested/returned versions and completed records;
unknown transport retryability remains explicitly unknown in that public error
contract. Decoding, Registry errors, invalid evidence, response-size violations,
chunk integrity failures, and agreement mismatches are not retried.

The pinned ic-agent 0.49.2 transport already retries HTTP 429/503 up to six
times internally. Those statuses are not retried again at the catalog layer.
The catalog count measures explicit query invocations, including added retries;
it does not count ic-agent's internal HTTP attempts or ancillary verification
requests. This matches the preexisting meaning of that count. Added backoff is
cancellable and has no detached work. Call duration is still transport-bound;
callers should retain an overall timeout.

## Downstream integration

Canic can continue using its existing request, assurance floor, deadline,
heartbeats, cache policy, and typed failure projection. To use history reuse and
structured progress:

1. Retain one `LiveSubnetCatalogSource` across acquisitions and immediate retry.
2. Construct it with `with_progress` and send the events to the existing progress
   observer; retain the independent ten-second heartbeat during slow calls.
3. Pass it to `load_subnet_catalog_detailed_with_source_async` inside the existing
   shared deadline. Dropping that future releases owned collection work and the
   refresh lock. Do not recreate the source for every retry.

For separate processes, construct the source with the same private root:

```rust
let source = LiveSubnetCatalogSource::with_progress(observer)
    .with_history_cache(cache_root);
```

Canic's caller-owned source must opt in using its existing private catalog root;
passing a source to a `*_with_source` API never silently reconfigures it. No
Canic files are changed by this upstream implementation.

The new public `SubnetCatalogProgressPhase::HistoryCache` variant is a hard
API cut for exhaustive progress matches. Canic's current
`crates/canic-host/src/subnet_catalog/ops/mod.rs::registry_progress` match must
handle this phase when adopting the release, and its client must configure the
root before collection. The phase reports local checkpoint handling, not a new
snapshot authority or freshness claim.

No sibling repository changes are needed upstream. No production CLI options
are added.

Canic currently constructs and drops its catalog client during a generation.
Its existing memory-only source therefore cannot help ordinary separate CLI
runs until it adopts the explicit disk configuration above. The upstream
convenience APIs already use the request's cache root for cross-process reuse.

## Cross-process qualification: 2026-10-02

The schema-1 disk implementation passes 1,151 workspace tests across all
targets/features and 212 library tests with Canic's minimal
`subnet-catalog-host` feature selection. Fixtures exercise separate OS
processes, endpoint/prefix isolation, tombstones, boundary pins, restored
cumulative limits, bounded file/identity retention, invalid-content repair,
confinement failures, writer-lock policy, and cancellation with batched durable
prefixes. Warning-denied pure, workspace, and minimal-feature Clippy, Rust
1.91.0 MSRV, feature boundaries, type/public documentation, process boundaries,
schema versions, formatting, and CI-script checks passed. The working-tree
0.44.2 ledger and detailed heading are prepared; the changelog gate also
requires committed HEAD entries, which remain the maintainer's release step.
Canic's tests, the full release gate, and live transport-fault injection were
not run.

Initial development-build timing exposed expensive per-page transcript
rewrites: a cold agreement took 318.832 s. Publishing the first page, every
eight pages, and completion reduced the repeated cold trial to 92.236 s with
the same 318 explicit queries. These are individual observations, not a
controlled benchmark or latency guarantee. An earlier five-minute trial was
terminated after durably reaching version 63722 at both endpoints. After
manually removing only that stopped trial's temporary writer locks, a new
process resumed with 168 queries, including four remaining history pages;
production lock recovery remains manual.

The final batched trial used `https://ic0.app` and `https://icp-api.io`, with
an empty private temporary root and Registry version **64518**. Cold agreement
took **92.236 s** and **318 queries**; cache-only reuse took **19.956 ms** and
returned exactly the same snapshot authority. A new source using the saved
disk history took **19.277 s** and **164 queries**, avoiding all **154 history
queries** while retaining fresh pinned value reads. Both acquisitions agreed
on canonical content digest
`5e7a98f27b9f4ec8a8ca797d564a595fba811107f9bb361d2fb30821e0e433ea`.
The owner-private `0600` transcript retained 77 normalized pages per endpoint
in 30,382,936 bytes, below the 64 MiB ceiling. The root and managed directories
remain `0700`. These upstream example timings are not Canic generation timings.

A second OS process using the same root reported restored prefixes through
version 64518 at both endpoints. Its first forced live agreement took
**20.979 s** and **164 queries**, avoiding all 154 cold history queries. Its
cache-only read took **21.029 ms** with identical snapshot authority, and its
fresh-source refresh took **21.012 s** and 164 queries. Every acquisition in
both processes returned the same Registry version and agreement digest. No
source object or memory checkpoint was shared between the two processes.

## Measurement

Use the explicit live developer example with an empty cache directory:

```bash
cargo run -p ic-query-cli \
  --example subnet_catalog_timing -- /tmp/icq-catalog-timing
```

The current example forces agreement from `https://ic0.app` and
`https://icp-api.io`, loads the resulting cache without network work, then
forces another refresh with a newly constructed source using disk history.
Run it again with the same directory to measure another process's first live
acquisition from the retained prefix. Use a new directory for each cold
measurement. Compilation is excluded from reported wall times. Historical
measurements below used the then-current memory-only example and are not
measurements of this new transcript implementation or latency guarantees.

Measured on 2026-09-15 against Registry version **64108**:

| Operation | Before | After | Explicit query calls |
| --- | ---: | ---: | ---: |
| Cold two-endpoint agreement | 158.961 s | 91.008 s | 316 → 316 |
| Immediate cache reuse | 19.294 ms | 18.701 ms | 0 → 0 |
| Forced refresh with retained source | Not measured | 18.181 s | 162 |

Cold acquisition was approximately 43% faster in this run. The retained-source
refresh avoided 154 history-page queries; it still performed all pinned value
reads. Both live runs and the retained-source refresh agreed on canonical
content digest `57265a09cae2112e237ed94758f7a96a2ad8e182ae63736a1f5f9b8973b1181b`.
Each immediate cache hit returned exactly its acquisition's snapshot authority.
The catalog digest can differ between separate acquisitions because it also
binds their provenance, including collection time and query counts. The
canonical agreement digest is the cross-acquisition content comparison.

The maintainer's earlier 148.245 s cold / 3 ms cache measurement is a separate
environment observation; these development-build cache timings do not establish
a regression or a production cache speedup. No live transport fault was
injected. Retry and cancellation behavior was checked with fixtures.

### 2026-09-25 cold-start follow-up

A fresh cache directory and a new process running the same development-build
example against `https://ic0.app` and `https://icp-api.io` produced:

| Operation | Wall time | Explicit query calls |
| --- | ---: | ---: |
| Cold two-endpoint agreement at Registry version 64346 | 88.199 s | 312 |
| Immediate complete-cache reuse | 18.382 ms | 0 |
| Forced refresh with the same in-memory source | 15.278 s | 158 |

The cache hit preserved snapshot authority. The retained-source refresh
avoided 154 history-page queries, while the cold run still reconstructed
history from version zero. These observations measure the upstream example,
not a Canic generation or separate Canic CLI runs. They support measuring the
downstream cold path before choosing a persistent-checkpoint design; they do
not establish a correctness defect or a general latency guarantee.

## Canic feedback review: 2026-10-02

This review preceded the cross-process implementation qualified above. Its
memory-only findings and proposed next slice describe that earlier checkout;
the current upstream transcript contract supersedes that proposal. Downstream
adoption still requires an explicitly configured caller-owned source.

Reviewed Canic's `ic-query-0.43.md` and `ic-query-0.43.1.md` reports under
`docs/audits/reports/2026-09/2026-09-15/`, its September 21 deployment-timing
report, the 0.110.42 dependency-update qualification, and its current catalog
consumer. Canic now selects published `ic-query` 0.44.0 with only
`subnet-catalog-host`.

| Feedback | Current outcome |
| --- | --- |
| Sequential endpoint collection exceeded the original 90-second deadline | Collection overlaps two endpoints; Canic retains its own shared 600-second deadline |
| Slow collection lacked useful progress | Endpoint, pin, history, record, and retry events feed Canic's bounded progress snapshots and ten-second heartbeats |
| Transient failures repeated expensive history work | Bounded retries preserve request inputs; a retained source resumes completely validated endpoint-local prefixes |
| Cancellation must preserve old authority and permit immediate retry | Fixture regressions cover both active endpoints, unchanged cache bytes, lock release, and immediate retry |
| Cold or expired-cache refresh in a new CLI process still scans history from zero | Open performance follow-up; checkpoints remain memory-only |

Canic's 0.43.1 production-client sample recorded 63.253 s for cold agreement,
3 ms for a cache hit, and 13.851 s for a live refresh with the retained client.
That refresh used a simulated expiry and avoided 154 history queries. These
are separate observations from this repository's measurements, not matched
benchmarks or evidence of a later 0.44 speedup. Canic's 0.110.42 qualification
reports ten passing host catalog regressions after adopting 0.44.0 and no
new live acquisition sample.

The current generation path constructs `MainnetCatalogClient::default()` for
one acquisition and drops it afterward. The source can retain validated
history during that acquisition and its stronger-assurance repair, but neither
a later generation nor another CLI process inherits it. The remaining
feedback therefore concerns cold-start work rather than a missing integration
of the reusable-source API. No additional catalog blocker was identified in
this review.

The next performance slice should qualify cross-process history reuse before
implementing a persisted checkpoint contract:

- Measure the actual Canic cold path, cache-hit path, and expired-cache path
  separately; attribute history and record query counts as well as wall time.
- Define explicit endpoint, network, Registry-canister, prefix, and version
  binding; retain tombstones and cumulative resource accounting. For an older
  requested pin, discard checkpoint reuse and replay from zero.
- Define the trust and integrity model for restored history. A serialized
  membership map or a self-recomputed digest does not prove that the retained
  history was complete or authenticate its contents.
- Specify confined ownership, byte and entry ceilings, atomic publication,
  concurrent writers, cancellation, and invalid-content recovery. Keep any
  new persisted schema at 1 under the pre-1.0 contract.
- Continue fetching current pinned records independently from each endpoint
  and require exact version/content agreement. Checkpoints must not become
  catalog authority, a freshness exemption, or shared endpoint evidence.

Live transport-fault injection remains an evidence gap. Retry and cancellation
fixtures establish the local policy, but do not qualify real transport-failure
latency or endpoint independence.

Review validation with Canic's exact `subnet-catalog-host` selection passed on
Rust 1.99.0: 78 catalog/source tests, 15 Registry transport tests, nine public
catalog API tests, and all-target Clippy with warnings denied. Whitespace
checks passed. Canic was inspected read-only; its tests and a fresh live
acquisition or transport-fault experiment were not run for this review.

## Canic output-alias follow-up: 2026-10-02

This section records the original reproduction and downstream recheck.
Current upstream code includes the fix and also protects Registry history
transcript and writer-lock aliases; release-state statements below describe
the checkout examined at the time.

Canic's updated `docs/status/current.md` records a separate correctness defect
against 0.44.0. Its retained `target/review-validation/ic-memory-0152-feedback.log`
and `ic-memory-0152-upstream-probe.rs` show a catalog dry run with the managed
catalog as its output path changing valid cache bytes while reporting
`wrote_catalog=false`. A distinct-output control preserves those bytes. This
feedback arrived after the acquisition-focused review above; the shared export
writer in released 0.44.1 still has the same defect.

The local regression reproduced the missing rejection on released code. The
fix rejects output aliases of the operation's managed catalog/cache and refresh
lock through the shared cache-file boundary. Catalog requests validate before
source collection, directory creation, or lock acquisition. The writer checks
again after collection and opens without truncation before comparing the opened
file identity. Resolved paths cover relative names, parent traversal, symlinks,
and missing targets; Unix file identities additionally cover hard links.

Fixtures cover both dry-run and normal publication, protected cache and lock
paths, missing targets, direct and indirect aliases, aliases introduced during
source collection, unchanged existing cache bytes, lock cleanup, and distinct
dry-run exports. Canic was inspected read-only. No sibling dependency change,
upstream issue posting, package version bump, or live acquisition was performed.

Validation passed on Rust 1.99.0: all 1,133 workspace tests across all targets
and features, 193 library tests with Canic's minimal `subnet-catalog-host`
selection, warning-denied Clippy for that selection, the pure library and full workspace,
the declared Rust 1.91.0 MSRV check, formatting, type-doc, library process-IO,
schema-version, and whitespace checks. Existing local HTTP fixture tests needed
a rerun with loopback access after the sandbox rejected their socket binds.
Live acquisition, transport-fault injection, Canic's tests, and the release gate
were not run for this follow-up.

Canic's subsequent upstream recheck in `docs/status/current.md` and
`docs/code-review/status.md` now explicitly selects registry 0.44.1 and tracks
the alias fix as present only in this uncommitted working tree. Its manifest,
lockfile, and cached release source confirm that disposition: the released
writer still uses `File::create`. Canic's current loader supplies no export
path, so the triggering combination is not exposed by that integration. No
new `ic-query` correctness feedback or live timing evidence was added;
cross-process history reuse remains open. All ten output-related regressions
passed again with Canic's minimal feature selection. This recheck did not
rerun the full workspace gate, Canic tests, or live acquisition, and made no
source, dependency, or release changes.

## Original acquisition-change validation

Focused checks passed:

- 78 catalog/source tests with `host`: agreement/version/content rejection,
  assurance floors, typed failures, cache authority, atomic publication,
  concurrent cancellation, lock cleanup, and immediate retry.
- 28 Registry transport tests: history continuity and deletion semantics,
  endpoint/pin isolation, checkpoint cancellation and recovery, post-pin
  mutations, chunk integrity, retry classification, attempt limits, and
  cancellation during backoff.
- 10 catalog/certified-catalog public API tests, including a cache hit with a
  reusable progress source that emits no events and preserves authority.
- The existing loopback response-size guard test (rerun with loopback permission
  after the sandbox initially rejected its socket bind).
- Portable library check without default features, formatting, and Clippy with
  warnings denied for the host library and CLI targets.

The full workspace test suite and live fault injection were not run. No package
versions, releases, tags, or published artifacts were changed.
