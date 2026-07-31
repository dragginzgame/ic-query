# IC Dashboard Canister Reporting

## Status

- Status: implemented
- Authority: official IC Dashboard REST API
- Collection mode: bounded live lookup, count, or cursor page
- Public family: `ic_query::ic`

## Decision

The `icq ic canister` family queries official Dashboard resources through one
`LiveIcSource` adapter:

- `info <canister-id>` uses `/api/v3/canisters/{canister_id}`;
- `count` uses `/api/v4/canisters/count`;
- `page` uses `/api/v4/canisters` with fixed canister-id ordering and an
  explicit limit of at most 100.

The public `IcCanisterSource` and `IcCanisterCollectionSource` capabilities let
fixtures, mirrors, and proxies reuse the same validation, projection, and
rendering paths.

The Dashboard is an official read-only analytics authority. It is not the
Registry, a certified canister response, or a management-canister status
query. Reports therefore identify `official_ic_dashboard_api` as their
authority and state both:

- `certified: false`;
- `point_in_time_guaranteed: false`.

Dashboard fields never inherit a Registry version or certified canister
authority.

## Detail Report Contract

The report preserves:

- the canonical requested canister principal;
- the Dashboard database row id and raw optional canister classification;
- raw name and language strings, including empty strings;
- canonical Subnet and controller principals;
- the raw current module hash, including an empty string when unavailable;
- the raw Dashboard `updated_at` value;
- nullable proposal-linked upgrade history with raw execution seconds, module
  hashes, and proposal ids;
- the API base endpoint, retrieval timestamp, and collector.

Controller rows are canonically ordered. Upgrade rows are ordered newest first
by execution timestamp and proposal id. Duplicate controllers, duplicate
upgrade proposal ids, non-canonical principals, malformed non-empty module
hashes, mismatched requested identity, and mismatched source provenance are
typed source-data failures rather than silently repaired evidence.

The live JSON decoder accepts additional response fields so an additive
Dashboard extension does not break existing reports, while required current
fields and their value types remain enforced.

## Count and Page Contract

Count and page share the official `has_name`, Subnet, controller, language,
canister-type, and text-query filters. Filter principals are canonicalized;
repeated raw language and canister-type values are ordered and must be unique;
and Dashboard text search is restricted to the documented two-through-100
character range before a source call.

Count returns only the filtered total. Page returns a discovery projection of
at most 100 rows with raw Dashboard classification, name, language, module
hash, update timestamp, Subnet, and controller tuple metadata. It deliberately
does not copy per-row upgrade history; the stable canister principal drives an
explicit `info` follow-up when that detail is required.

Page ordering is fixed to `canister_id`, so `after`, `before`,
`previous_cursor`, and `next_cursor` are canonical canister principals. The
two input directions are mutually exclusive. Custom sources must preserve the
requested filters, limit, cursor, endpoint, and collection provenance; return
no more than the requested rows; and return unique canister and Dashboard ids
in strict canister-id order. Row principals, controller uniqueness, non-empty
timestamps, optional module hashes, and boundary cursors are validated before
projection.

## User-Facing Usage

```bash
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --format json
icq ic canister count --has-name true
icq ic canister page --query ledger --limit 25 --format json
icq ic canister page --after ryjl3-tyaaa-aaaaa-aaaba-cai --limit 25
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai \
  --source-endpoint https://ic-api.internetcomputer.org/api/v3
```

Text output is a compact current-state view. JSON preserves raw nullable
classification and upgrade history, full principals and hashes, Dashboard and
retrieval timestamps, and the explicit authority guarantees. Command help is
the definitive option reference; the broader command map is in
[CLI Usage](../cli-usage.md).

## Endpoint and Network Contract

The default detail endpoint is:

```text
https://ic-api.internetcomputer.org/api/v3
```

The default count/page endpoint is:

```text
https://ic-api.internetcomputer.org/api/v4
```

The base endpoint must be an HTTP(S) URL with a host and without a query or
fragment. The canister path is appended only after the request principal has
been parsed and canonicalized.

The official API represents mainnet and the report records `network: ic`.
The `ic` command family uses endpoint plus canister identity and rejects the
top-level `--network` option before dispatch. Endpoint overrides remain
explicit through `--source-endpoint`.

## Cache Contract

Every canister operation makes exactly one bounded current-state REST lookup.
Count fetches no rows. Page defaults to 50 rows, is capped at 100, and never
automatically follows a returned cursor. The family is live-only and does not
read, write, invalidate, or migrate cache files. A future explicitly requested
complete Dashboard collection or time-series report would need a separate
design, operational cap, timestamped snapshot identity, and API endpoint
provenance; it must not reuse Registry topology caches.

## Scope

This report does not:

- enumerate, automatically page, or cache the complete canister collection;
- call the management canister;
- prove controller or module state cryptographically;
- infer a canister type when the Dashboard returns `null`;
- replace raw IC terminology with a downstream management classification;
- add boundary-node, replica, throughput, cycle-burn, block-rate, energy, or
  trustworthy-metric reports.
