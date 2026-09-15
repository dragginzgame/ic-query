# Subnet Catalog acquisition performance

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
`*_with_source` APIs. Its clones share memory-only history checkpoints and an
optional structured progress callback. Keep one instance across retry and
refresh attempts. The stateless convenience APIs also get endpoint overlap and
the bounded retry policy, but do not retain history between loads.

## History reuse and authority

Checkpoints are keyed by exact endpoint string and key-family prefix, with at
most eight retained entries per source. Each contains the membership state
(including tombstones), highest completely validated version, and cumulative
resource counters. There is no global cache, disk sidecar, schema change, or
checkpoint serialization API.

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

Cold collection in a new process still scans history from zero. Persisting
checkpoints would require a separate integrity/ownership contract and recovery
policy. That additional contract is intentionally deferred. Cumulative
resource ceilings remain conservative, including values beyond the pin that
were inspected in a boundary page.

## Progress and retries

`LiveSubnetCatalogSource::with_progress` accepts a synchronous `Send + Sync`
callback receiving `SubnetCatalogProgress`:

- exact endpoint and endpoint-local query-attempt count;
- endpoint start and pinned Registry version;
- history watermark, target version, and whether the prefix was reused;
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

No sibling repository changes are needed upstream. No production CLI options
are added.

## Measurement

Use the explicit live developer example with an empty cache directory:

```bash
cargo run -p ic-query-cli \
  --example subnet_catalog_timing -- /tmp/icq-catalog-timing
```

The example acquires agreement from `https://ic0.app` and `https://icp-api.io`,
loads the resulting cache, then forces a refresh using the same source to
measure history reuse. Use a new directory for each cold measurement. The
before measurement used the same first two operations with the original
collector; compile time is excluded. These are single-run development-build
wall times, not a latency guarantee.

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

## Validation

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
