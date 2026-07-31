# IC Dashboard Canister Reporting

## Status

- Status: implemented
- Authority: official IC Dashboard REST API
- Collection mode: bounded live lookup
- Public family: `ic_query::ic`

## Decision

`icq ic canister info <canister-id>` queries the official Dashboard
`/api/v3/canisters/{canister_id}` resource through the `LiveIcSource` adapter.
The public `IcCanisterSource` capability lets fixtures, mirrors, and proxies
reuse the same validation, projection, and rendering path.

The Dashboard is an official read-only analytics authority. It is not the
Registry, a certified canister response, or a management-canister status
query. Reports therefore identify `official_ic_dashboard_api` as their
authority and state both:

- `certified: false`;
- `point_in_time_guaranteed: false`.

Dashboard fields never inherit a Registry version or certified canister
authority.

## Report Contract

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

## User-Facing Usage

```bash
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --format json
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai \
  --source-endpoint https://ic-api.internetcomputer.org/api/v3
```

Text output is a compact current-state view. JSON preserves raw nullable
classification and upgrade history, full principals and hashes, Dashboard and
retrieval timestamps, and the explicit authority guarantees. Command help is
the definitive option reference; the broader command map is in
[CLI Usage](../cli-usage.md).

## Endpoint and Network Contract

The default endpoint is:

```text
https://ic-api.internetcomputer.org/api/v3
```

The base endpoint must be an HTTP(S) URL with a host and without a query or
fragment. The canister path is appended only after the request principal has
been parsed and canonicalized.

The official API represents mainnet and the report records `network: ic`.
The `ic` command family uses endpoint plus canister identity and rejects the
top-level `--network` option before dispatch. Endpoint overrides remain
explicit through `--source-endpoint`.

## Cache Contract

Canister detail is one bounded current-state REST lookup, so this slice is
live-only. It does not read, write, invalidate, or migrate cache files. A
future Dashboard collection or time-series report may add timestamped
snapshots under its own identity and must not reuse Registry topology caches.

## Scope

This report does not:

- enumerate or search the complete canister collection;
- call the management canister;
- prove controller or module state cryptographically;
- infer a canister type when the Dashboard returns `null`;
- replace raw IC terminology with a downstream management classification;
- add boundary-node, replica, throughput, cycle-burn, block-rate, energy, or
  trustworthy-metric reports.
