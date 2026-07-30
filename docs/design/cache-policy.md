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
- Commands whose complete snapshots can be expensive or require user-controlled
  page limits may require an explicit refresh before cache-backed reads.
- Live network calls must remain visible in output when a command refreshes or
  creates a cache.
- Cache keys describe collected data, not view options. Sorting, limits,
  verbosity, and text formatting must not create separate complete snapshots.
- Failed refreshes should not replace a previously complete cache.

## Shared Missing-Cache Flow

Cache-backed reads should follow this sequence:

1. Try to load the complete cache.
2. If the cache is missing, print the standard refresh announcement with the
   component name, cache path, and source endpoint.
3. Refresh or create the cache through the command-owned refresh path.
4. Load the cache again and build the report from the cached data.

Errors other than a missing cache are not refresh triggers. Parse failures,
schema mismatches, network mismatches, and IO failures should be reported to
the user instead of silently replacing local state.

Use `cache_file::load_or_refresh_missing_cache` for this policy when the
command already has:

- a cache loader
- a refresh implementation
- a typed missing-cache error that contains the expected cache path

## Manual Refresh

Manual refresh commands always refresh explicitly and should report refresh
progress or status through their owning report modules. They do not need the
missing-cache helper because the user has already requested refresh behavior.

## Refresh Locks

Refresh locks are never removed automatically. Parsed locks older than the
command's stale-lock threshold are reported explicitly as stale; malformed
locks are reported as invalid. Commands show the lock path and require the
operator to remove it manually after verifying that no refresh is still
running. This avoids deleting a newly acquired lock during concurrent stale
lock recovery.

## Cache Discovery

Cache status and cache list commands should inspect local state only. They
should discover complete full-collection snapshots through the shared
snapshot-cache path scanner so cache listing and id lookup behavior stays
deterministic across command families.
Complete snapshot caches carry required logical identity fields and are
validated against the expected cache key on load. Identity-less snapshots are
unsupported and require an explicit refresh.
Cache status and cache list commands should render malformed, unsupported, or
identity-mismatched local snapshot files as invalid local cache rows instead of
silently ignoring them or making live calls. Normal cache-backed report reads
should still reject those invalid snapshots.

## Current Coverage

The shared missing-cache flow is used by:

- subnet catalog loads
- cached NNS node, node-provider, node-operator, and data-center list reports
- SNS proposal list auto-cache creation

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
