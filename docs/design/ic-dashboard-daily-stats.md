# IC Dashboard Daily Statistics

## Status

- Status: implemented
- Authority: official IC Dashboard REST API
- Collection mode: one bounded live daily-statistics lookup
- Public family: `ic_query::ic`

## Decision

`icq ic network daily-stats` queries the official v3 `daily-stats` resource
through the existing `LiveIcSource` and `IcNetworkSource` capability. It
selects the resource's network-activity fields rather than copying its entire
cross-domain payload into a second reporting authority.

The selected fields are:

- `average_query_transactions_per_second`;
- `average_update_transactions_per_second`;
- `average_transactions_per_second`;
- `max_query_transactions_per_second`;
- `max_update_transactions_per_second`;
- `max_total_transactions_per_second`;
- `blocks_per_second_average`.

Governance, token-supply, topology, Internet Identity, and other additive
daily fields remain outside this report. Those subjects either already have a
more direct authority in `ic-query` or require a separate reporting contract.

## Request and Size Contract

Every query supplies inclusive Unix-second `start` and `end` bounds. The CLI
defaults to the preceding seven days ending at collection time. Builders:

- reject starts earlier than the API minimum, `1620406800`;
- reject an end before the start or later than collection time;
- cap the window at 366 days;
- accept no more than 366 returned daily rows.

The live source validates the window before parsing the endpoint. It makes one
JSON request containing `format=json`, `start`, and `end`. It does not
paginate, follow another endpoint, expand per Subnet, or query another
resource.

## Report Contract

`IcDailyStatsReport` preserves:

- the exact requested start and end;
- each raw Dashboard day and Unix timestamp;
- each selected rate as its raw string;
- the returned daily-row count;
- endpoint and retrieval provenance.

Rows are canonically ordered by increasing timestamp. Projection permits
missing calendar days but rejects rows outside the requested window, duplicate
days or timestamps, a day that does not match its timestamp's UTC date, empty
or invalid rate text, negative or non-finite rates, more than 366 rows, a
mismatched custom-source query, and mismatched source provenance.

Rate strings are parsed only for finite/nonnegative validation and are not
converted in the report. This preserves the Dashboard's precision and lexical
representation for downstream consumers. The live decoder requires every
selected field while tolerating additive row and response fields.

## Authority and Freshness

The default endpoint is:

```text
https://ic-api.internetcomputer.org/api/v3
```

The endpoint must be HTTP(S), include a host, and contain no query or fragment.
The report uses the shared Dashboard provenance contract:

- `network: ic`;
- `authority: official_ic_dashboard_api`;
- `certified: false`;
- `point_in_time_guaranteed: false`.

The values are off-chain daily analytics. They do not inherit Registry,
certified state-tree, ledger, or native Governance authority, and distinct
daily rows are not presented as one IC state snapshot.

## User-Facing Usage

```bash
icq ic network daily-stats
icq ic network daily-stats \
  --start 1784937600 --end 1785542400
icq ic network daily-stats --format json
```

Command help is the definitive option reference.

## Cache Contract

Daily-statistics reports are live-only. They do not read, write, invalidate,
or migrate cache files. The explicit bounds and 366-row cap prevent the
command from becoming an implicit whole-history collector.

## Scope

This report does not:

- expose every field in the broad Dashboard daily-statistics payload;
- issue per-Subnet, per-node, or per-canister calls;
- infer values for missing days;
- query an unbounded history;
- claim certified or point-in-time-complete network evidence;
- create or update a cache.
