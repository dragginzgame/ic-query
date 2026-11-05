<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-query/ic-query-readme-header.svg" alt="IC Query — a read-only Internet Computer explorer">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->

# CLI Usage

`icq` exposes read-only reports grouped by the authority or protocol family
that supplies them. This guide describes the stable command hierarchy and
collection behavior. Run a command with `--help`, or use `icq help <path>`, for
its complete current option reference. A command namespace without its next
operation displays the same complete local help as its explicit `help`
subcommand. Incomplete leaf operations and invalid values remain errors. Every
`Commands` section is ordered alphabetically by command name, including the
generated `help` entry.

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-query/ic-query-cli-authority-map.svg" alt="Map from IC Query command families to certified IC state, Registry, canisters, ledgers, indexes, archives, control-plane services, and official Dashboard sources">
</p>

## Common behavior

Human text clips long summaries and metadata to terminal-column budgets with an
`…` marker. Raw JSON retains the complete strings. NNS Governance, SNS neuron
and ICRC history cache status use common `attempt_*` labels for lifecycle,
progress, cursor and failure evidence; ICRC includes `attempt_index_canister_id`.


```bash
icq help
icq help nns
icq sns
icq nns topology summary --help
```

Report commands use human-facing text by default and accept
`--json` for raw, script-friendly output. JSON reports retain stable
identifiers, native numeric fields, source endpoints, collection timestamps,
and authority guarantees.

Report and persisted schemas are versioned independently and currently use
`schema_version` value `1`. Before 1.0, incompatible shape changes replace the
previous contract in place without aliases, legacy readers, or migrations.

Inspect all known complete caches under the selected user-level root without a
network call or mutation:

```bash
icq cache status
icq cache status --json
```

The report separates generic header integrity from age. It labels only caches
with an explicit age policy as `fresh` or `stale`; `unmanaged` means a valid
age has no registered threshold, while missing or invalid timestamp evidence
has `unknown` age. It also shows the canonical owner's invalid-content
recovery policy as `automatic`, `explicit`, `missing_only`, or `unknown`.
Because the report spans all cached networks, `icq cache` rejects `--network`.

## Target identity

CloudEngine, NNS, SNS, and system-canister commands accept the global network
identity:

```bash
icq --network ic cloud-engine prices
icq --network ic nns registry version
icq --network ic sns neuron list "$GOVERNANCE_CANISTER_ID" --json
icq --network ic system xdr
```

The built-in sources and caches currently support only mainnet, named `ic`.
A different network is rejected before a live adapter is constructed.

Official Dashboard canister and ICRC commands identify their target using an
entity principal plus `--source-endpoint`. Dashboard metric and network
resource commands use an official resource identity and `--source-endpoint`.
These families reject the top-level `--network` option instead of silently
ignoring it. Every override must be a credential-free HTTP(S) base URL with a
host and no query or fragment. Official Dashboard requests reject redirects;
native `ic-agent` calls cap each response body at 8 MiB.
`cloud-engine node` and `cloud-engine provider` remain under the top-level
CloudEngine family and therefore honor the global network identity, currently
only `ic`; their `--source-endpoint` selects the official Dashboard authority.

The certified API boundary-node command is also fixed to mainnet and rejects
the top-level `--network` option. Its `--source-endpoint` is the mainnet IC API
endpoint used for authenticated `read_state`; the fixed Registry effective
canister id routes the request but is not the report's data authority.

The node, Subnet, and node-provider `status` operations sit below their NNS
nouns so discovered Registry principals flow directly into the observed view.
They accept the global network identity, require it to be `ic`, and query the
explicit Dashboard `--source-endpoint` only when their shared cache policy
requires refresh.
Node-provider `reward` operations share that NNS noun but use the separate
official Dashboard v3 `--source-endpoint`; they are mainnet-only live reports
and never reuse the Registry inventory or node-status caches.

## CloudEngine

```bash
icq cloud-engine list
icq cloud-engine list --json
icq cloud-engine info 2nl67-oqoc5-cmocj-otlhq-kr2kr-53hov-drrds-7ihcs-fhomv-2eyvu-6qe
icq cloud-engine info 2nl67-oqoc5-cmocj-otlhq-kr2kr-53hov-drrds-7ihcs-fhomv-2eyvu-6qe --json
icq cloud-engine prices
icq cloud-engine prices --json
icq cloud-engine node list
icq cloud-engine node list --node-provider-id bvcsg-3od6r-jnydw-eysln-aql7w-td5zn-ay5m6-sibd2-jzojt-anwag-mqe
icq cloud-engine node info 53amq-7hjxu-6lxaj-o2sp6-kmngy-qa22h-b7bo6-oeyyn-fkqnv-7tauf-7qe --json
icq cloud-engine provider list
icq cloud-engine provider list --json
icq cloud-engine provider info rbn2y-6vfsb-gv35j-4cyvy-pzbdu-e5aum-jzjg6-5b4n5-vuguf-ycubq-zae
```

`list` obtains the complete Registry-classified CloudEngine Subnet inventory
through the existing Subnet Catalog missing-or-invalid cache policy, then
attempts one exact `getEngineOperatorBySubnet` query per returned row, capped
at 100. It preserves the Registry version, assurance, catalog provenance, and
cache disposition separately from the control-plane endpoint, timestamp, and
uncertified consistency limits. Each row distinguishes a resolved binding, a
successful absent binding, and a failed lookup; one failed lookup does not
discard the Registry inventory. `--registry-source-endpoint` selects the
endpoint authorized for a needed catalog refresh, while `--source-endpoint`
selects the control-plane query endpoint.

`info` resolves exactly one Subnet through the fixed mainnet CloudEngine
control-plane canister. An absent binding is a successful one-call result and
does not prove that the Subnet or engine exists. When the response has an
operator, the command follows it with four public
operator queries for owner, platform administrator, Caffeine setting, and
claimed domains. The report therefore records an exact query count of one or
five. It accepts at most 100 unique claimed domains and never enumerates other
CloudEngine Subnets.

`prices` makes exactly two control-plane queries: one for the network-fee
fraction and one for at most 1,000 marketplace rows. JSON retains raw net and
gross monthly cycle amounts as arbitrary-precision decimal strings accepted up
to 256 digits, the native flattened row key, structured
node/data-center/provider identity, and signed
Unix-nanosecond update timestamp. Text formats the cycle amounts for people.

`provider list` makes one official Dashboard request for the complete node-
provider resource, validates every row up to 1,000, and then retains providers
with a nonzero CloudEngine count or at least one CloudEngine location. It
reports both the source and retained counts. `provider info` makes one exact
request for an NNS node-provider principal and preserves all raw general and
CloudEngine counts and locations. A valid row with no CloudEngine fields is
returned with `cloud_engine_evidence_present: false`; the target is not a
Subnet, operator canister, or engine canister.

`node list` makes one official Dashboard request with the explicit `Type4`
reward filter plus repeated `DOWN`, `UP`, `DISABLED`, and `DEGRADED` status
filters. It accepts at most 10,000 rows and may apply one exact
`--node-provider-id` filter remotely. The report preserves raw status,
provider/operator, ordinary and CloudEngine Subnet assignment, alert,
GuestOS, connectivity, hardware, and location fields; a null
`cloud_engine_subnet_id` remains an unassigned observation. `node info` makes
one exact node-id request and requires the returned reward type to be `Type4`.

`info` and `prices` are mainnet-only bounded live queries. They never read or
write a report cache, and `--source-endpoint` keeps the replica endpoint
explicit. All three operations' ordinary control-plane responses state
`certified: false` and
`point_in_time_guaranteed: false`; sequential calls do not inherit a Registry
version or form one snapshot. See
[0.31 Public CloudEngine Reporting](design/0.31/0.31-design.md). Provider
reports are also live and uncached, preserve explicit off-chain Dashboard
provenance, and make no health or native control-plane claim. See
[0.34 CloudEngine Provider Reporting](design/0.34/0.34-design.md). Type4 node
reports remain live and uncached, do not reuse the default-scope status cache,
and do not claim that a separately timed provider aggregate agrees. See
[0.35 CloudEngine Type4 Node Reporting](design/0.35/0.35-design.md).

## Certified IC state

```bash
icq ic api-boundary-node list
icq ic api-boundary-node list --json
icq ic api-boundary-node list --source-endpoint https://icp-api.io
```

`api-boundary-node list` makes exactly one response-bounded mainnet
`read_state` request for the complete certified `api_boundary_nodes` subtree.
It authenticates the certificate against the built-in mainnet root key and
returns canonical node principals, DNS domains, optional IPv4 addresses, and
required IPv6 addresses. Every row belongs to the same certificate time.

JSON preserves the raw certified time as Unix nanoseconds, a direct
floor-to-seconds projection, and its UTC rendering. Text shows the ordinary
Unix-second value. The report states `authority: certified_ic_state_tree`,
`certified: true`, and `point_in_time_guaranteed: true`.

The command is live-only and uses no cache, pagination, endpoint discovery, or
per-node follow-up. These rows describe configured API boundary nodes. They do
not establish operational health, reachability, latency, data-center location
or ownership, node-provider identity, or membership in the separate HTTP
gateway fleet. See
[0.33 Certified API Boundary-Node Reporting](design/0.33/0.33-design.md).

## Official IC Dashboard

```bash
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai --json
icq ic canister count --has-name true
icq ic canister page --query ledger --limit 25 --json
icq ic canister page --after ryjl3-tyaaa-aaaaa-aaaba-cai --limit 25
icq ic canister info ryjl3-tyaaa-aaaaa-aaaba-cai \
  --source-endpoint https://ic-api.internetcomputer.org/api/v3

icq ic metrics instruction-rate
icq ic metrics cycle-burn-rate \
  --start 1700000000 --end 1700003600 --step 300
icq ic metrics ic-node-count --json

icq ic network boundary-node-data-centers
icq ic network boundary-node-data-centers --json
icq ic network daily-stats
icq ic network daily-stats \
  --start 1784937600 --end 1785542400

icq ic replica-version list --limit 25
icq ic replica-version list \
  --offset 25 --max-proposal-index 438 --json
icq ic replica-version info e3d101b22ae3fa02aca737f9fb96cc6c4ca83ac3
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
`--before` to another command. `metrics` selects one documented aggregate
network metric and preserves its raw named series, Unix timestamps, and
value strings. Its default window is the preceding hour at a
five-minute step; explicit windows are capped at 1,000 observations per
series. It does not fan out over metrics or Subnets.

`network boundary-node-data-centers` returns the official v4 data-center
aggregates in canonical id order. It preserves raw owner, region, coordinate,
and node-count strings, including zero-node locations, and derives only row and
node totals. The endpoint has no pagination parameters; the command makes one
request and no per-location follow-up calls.

`network daily-stats` selects the official v3 resource's daily network-activity
fields: raw average and maximum query, update, and total transaction rates plus
average block rate. It defaults to the preceding seven days and is capped at a
366-day window and 366 returned rows. Missing days remain missing; the command
makes one request and does not copy the resource's unrelated governance,
supply, topology, or Internet Identity fields into this report.

`replica-version list` selects one release page in descending execution-time
order. It defaults to 50 rows, caps the page at 100, and exposes the returned
proposal-index ceiling and next offset without following them automatically.
Supply `resolved_max_proposal_index` as `--max-proposal-index` on a later page
to retain the same upper indexing bound. `replica-version info` makes one exact
request and preserves the raw release summary and Dashboard-recorded Subnet
rollout proposals. Neither operation proves which replica binary a Subnet is
currently running.

There is no automatic enumeration and these commands never read or write a
cache. The official Dashboard is an off-chain analytics authority, so every
report states
`certified: false` and `point_in_time_guaranteed: false`. It does not inherit a
Registry version. Canister reports also do not prove current controller or
module state.

See [IC Dashboard Canister Reporting](design/ic-dashboard-canister-reporting.md)
and [IC Dashboard Network Metrics](design/ic-dashboard-network-metrics.md) for
the canister and metric contracts. See
[IC Dashboard Boundary-Node Reporting](design/ic-dashboard-boundary-node-reporting.md)
and [IC Dashboard Daily Statistics](design/ic-dashboard-daily-stats.md) for the
network-resource contracts. See
[0.32 Bounded Replica-Version Reporting](design/0.32/0.32-design.md) for release
pagination, detail, and authority boundaries.

## NNS

### Registry inventory and topology

```bash
icq nns registry version

icq nns subnet list
icq nns subnet info <subnet-or-canister>
icq nns subnet refresh

icq nns node list
icq nns node status
icq nns node status <node-or-prefix> --json
icq nns node-provider list
icq nns node-provider status
icq nns node-operator list
icq nns data-center list

icq nns subnet status
icq nns subnet status <subnet-or-prefix> --all

icq nns topology refresh
icq nns topology summary
icq nns topology coverage
icq nns topology check
icq nns topology gaps
icq nns topology capacity
icq nns topology regions
icq nns topology providers
icq nns topology versions
```

Inventory list/detail commands use their documented cache-backed behavior.
Refresh commands force a live fetch and replace the matching complete cache
only after validation.

`nns registry version` is a live, uncached, one-call certified query. It
authenticates the system certificate against the mainnet root key, verifies
the Registry mixed-hash-tree commitment and canonical `current_version` leaf,
and preserves certificate time and raw evidence in JSON. That certification
applies to the latest version value only; it does not certify the separate
ordinary-query Subnet Catalog or topology record reads.

The three `status` operations are views over one off-chain Dashboard node
snapshot, not joins performed independently for each noun. The default view
shows only non-up nodes or groups; `--all` includes fully-up rows, an exact
principal or unique prefix selects one target, and `--refresh` forces a live
replacement. Ordinary reads reuse the complete cache for up to 60 seconds and
visibly recover missing, malformed, semantically invalid, or stale content.
The source makes one unfiltered `/nodes` call, accepts at most 10,000 rows and
8 MiB, and performs no per-node or per-Subnet follow-up calls.

Status JSON preserves raw Dashboard `UP`, `DOWN`, `DEGRADED`, `DISABLED`, and
unknown future values, assignment/type evidence, alerts, provider/operator,
version, location, and hardware fields. Subnet rows derive separate down-only
and conservative non-up distances from `floor((n - 1) / 3)`. Provider rows
include a complete status-by-assignment cross-tab plus typed `less`, `equal`,
or `greater` outcomes for unassigned versus assigned up and conservative
non-up counts. Report-level totals group every observed provider by those
outcomes before view filtering. These are timestamped off-chain observations
with `certified: false` and
`point_in_time_guaranteed: false`; the recorded default public-mainnet scope
also excludes cloud-engine nodes.

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
The component-cache consistency operation is named `topology check` so it is
not confused with the observed operational status views.

### Node-provider rewards

```bash
icq nns node-provider reward info 7562
icq nns node-provider reward list --limit 25
icq nns node-provider reward list \
  --offset 25 --max-reward-index 6470 --json
icq nns node-provider reward history
icq nns node-provider reward history \
  --start 1752537600 --end 1784073600 --step 86400 --json
```

Each operation makes exactly one official Dashboard request and creates no
cache. `info` preserves one exact record id; raw e8s amount; canonical provider
principal; Unix-second reward time; separate raw Dashboard update string;
optional proposal, Registry, cap/floor, and XDR evidence; raw mode name; and
mode-specific JSON details.

`list` defaults to 50 rows, accepts at most 100, and never automatically
follows an offset. The returned maximum reward index may pin a later explicit
request, but live adjacent pages can contain overlapping ids even when pinned.
JSON therefore exposes `pages_may_overlap: true`, preserves source row order,
and calls the continuation only `next_offset_hint`; it does not claim API
exhaustion or a complete snapshot.

`history` defaults to the preceding 365 days at an 86,400-second step. Steps
must be from 60 through 259,200 seconds and the inclusive request may imply at
most 1,000 observations. Each returned aggregate is named explicitly as raw
`amount_e8s` plus `timestamp_unix_secs`.

All reward reports identify `official_ic_dashboard_api`, are off-chain and
uncertified, and state that they are not point-in-time guaranteed. See
[0.36 Node-Provider Reward Reporting](design/0.36/0.36-design.md).

### Governance

Bounded point-value reports are live-only:

```bash
icq nns governance economics
icq nns governance metrics --json
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

Complete snapshots also support local analytics:

```bash
icq nns proposal activity
icq nns proposal activity --from 1700000000 --until 1700100000 --json
icq nns neuron distribution
icq nns neuron distribution --json
```

These operations are cache-only: missing or invalid snapshots fail without a
network call or cache write. Run the matching explicit `refresh` first when
needed. Proposal activity counts retained proposals by topic, status, reward
status, and UTC creation day; its Unix-second window includes `--from` and
excludes `--until`. Neuron distribution reports public-neuron counts and raw
stake/optional-field totals with coverage. JSON uses the existing pure report
shapes, and source provenance describes the stored collection's transport.

Starting with 0.46.0, proposal and neuron snapshots require the recorded final
`collection_state`. Explicitly refresh existing snapshots before cache-backed
reads; the schema remains `1`, with no migration or fallback reader. See
[Cache-only NNS Analytics](design/0.46/0.46-design.md).

## System canisters

Cycle Minting Canister reports are bounded live point queries:

```bash
icq system xdr
icq system xdr --json
icq system cycles
```

Both commands make exactly one native mainnet CMC
`get_icp_xdr_conversion_rate` query. The host adapter authenticates the IC
system certificate against the CMC principal, verifies that the returned hash
tree is committed by the canister's `certified_data`, and proves that the
native `ICP_XDR_CONVERSION_RATE` leaf equals the returned Candid value.

`xdr` preserves the raw market-data timestamp and
`xdr_permyriad_per_icp`. `cycles` preserves that same certified input and
derives `cycles_per_icp` exactly as
`xdr_permyriad_per_icp * 1_000_000_000_000 / 10_000`, using the IC protocol
constant of one trillion cycles per XDR. The report carries the formula and
raw certificate/hash-tree evidence; text additionally formats the rate to four
decimal places without replacing the raw field.

The CMC public Candid interface does not expose total cycles minted. `icq`
does not scrape the CMC's explicitly uncertified Prometheus metrics, call
hidden methods, enumerate canisters, or create a cache. See
[Certified CMC System Reporting](design/cmc-system-reporting.md).

## SNS neuron ownership and stake

Configure the Governance canister that holds the existing neurons. Operations
use that explicit principal; SNS-W discovery, numeric project ids, Root lookup,
project catalogs, proposals, parameters, swaps, upgrades, metrics and canister
health reports are retired in 0.91. Use the generic ICRC ledger commands for token
metadata, balances and transaction/burn evidence. Indexed holder enumeration stays
under `icrc analytics holder`; its paginated results describe the selected index.

```bash
icq sns neuron list "$GOVERNANCE_CANISTER_ID" --owner "$USER_PRINCIPAL" --json
icq sns neuron info "$GOVERNANCE_CANISTER_ID" "$NEURON_ID" --json
icq sns neuron refresh "$GOVERNANCE_CANISTER_ID"
icq sns neuron list "$GOVERNANCE_CANISTER_ID" --sort stake --limit 1000 --json
icq sns neuron cache status "$GOVERNANCE_CANISTER_ID" --json
```

Live `list --sort api` reads one bounded page; owner filtering selects principals
with neuron permissions. A principal's presence alone does not establish purchase
eligibility. JSON retains every native permission code, including unknown codes,
for each principal. Toko owns the permission and stake policy.

Complete refresh pages until exhaustion, preserves inclusive cursor boundaries
once and rejects page ceilings before publishing. Cached sorts use only complete
local snapshots. Stake, neuron fees, maturity and staked maturity remain separate
raw values. Dissolve and vesting evidence is retained. Sequential scans can span
state changes; collection completeness does not imply one atomic snapshot.

The schema-1 hard cut uses `sns-neuron/<network>/<governance>/neurons/full.json`
and matching lock/attempt files. Old Root-addressed `sns/` files stay untouched;
there is no discovery, old reader, migration or automatic deletion. Explicitly
refresh the new Governance target before using a cached sort. Family cache status
and list remain local and preserve invalid-content and failed-attempt evidence.

Query supplies read-only evidence. Token acceptance, transfers, burns, idempotent
credits and NFT purchase decisions belong in Toko's application flow. Balance
reads address known accounts; holder lists page the configured Dashboard index.

## ICRC

Bounded official ICRC analytics:

```bash
icq icrc analytics account count mxzaz-hqaaa-aaaar-qaada-cai
icq icrc analytics account list mxzaz-hqaaa-aaaar-qaada-cai --limit 25
icq icrc analytics account list mxzaz-hqaaa-aaaar-qaada-cai \
  --sort-by=-balance --after '1668734888.0,hkmli-faaaa-aaaar-qb4ba-cai' --json
icq icrc analytics account info mxzaz-hqaaa-aaaar-qaada-cai \
  222nw-nqiei-h4uy6-fqm54-d3slu-jveav-vqrn6-yojxi-4eug3-2ejie-vae
icq icrc analytics holder count mxzaz-hqaaa-aaaar-qaada-cai
icq icrc analytics holder list mxzaz-hqaaa-aaaar-qaada-cai --sort-by=-balance
icq icrc analytics token-values mxzaz-hqaaa-aaaar-qaada-cai
icq icrc analytics token-values mxzaz-hqaaa-aaaar-qaada-cai \
  --start 1785542400 --end 1785628800 --limit 100 --json
icq icrc analytics total-supply mxzaz-hqaaa-aaaar-qaada-cai
icq icrc analytics total-supply mxzaz-hqaaa-aaaar-qaada-cai \
  --start 1785542400 --end 1785801600 --step 86400 --json
icq icrc analytics transaction count mxzaz-hqaaa-aaaar-qaada-cai
```

`analytics total-supply` makes one request to the official IC Dashboard ICRC
analytics API for one canonical ledger principal. It defaults to the preceding
30 days at a daily step; only hourly (`3600`) and daily (`86400`) steps are
accepted, and requested and returned series are capped at 1,000 observations.
It follows no cursor, performs no per-row or native-ledger follow-up, and never
reads or writes a cache. Values remain unsigned decimal strings in raw ledger
base units.

`analytics account|holder|transaction count` each makes one v2 request for
only the current unfiltered indexed total. These commands have no list limit,
cursor, filter, or page because they do not request indexed rows. Counts are
Dashboard-index values rather than scans performed by `ic-query`; the commands
never follow up against the ledger or create a cache. Account and holder counts
remain distinct API classifications.

`analytics account list` and `analytics holder list` each make one v2 request
for at most 100 rows. They preserve the selected upstream sort, make no hidden
enumeration, and expose the response's opaque `previous_cursor` and
`next_cursor`. A returned cursor can be supplied unchanged to `--before` or
`--after`; composite cursors are translated to the upstream repeated query
parameters only at the HTTP boundary. The commands never follow or merge
pages automatically and do not claim API exhaustion, completeness, or one
point-in-time snapshot. Account list can additionally restrict the one page
to a canonical owner principal.

`analytics account info` makes one v1 lookup for the exact opaque account id
returned by account list. Account rows preserve owner, raw subaccount and
base-unit balance, transaction counts, latest transaction index, raw Dashboard
update time, and fee-collector evidence. Holder rows preserve the principal-
level balance and transaction aggregate plus raw percentage and nullable USD
value. The upstream nanosecond creation value is represented as ordinary
`created_at_unix_secs` plus `created_at_subsec_nanos`, retaining full precision
without making callers interpret a 19-digit timestamp.

`analytics token-values` makes one v2 request for a bounded range of external
price and 24-hour trading-volume observations. It defaults to 24 hours and
1,000 rows, rejects ranges longer than 90 days or limits above 1,000, and marks
`limit_reached` when the response may be truncated. It preserves both raw
value field pairs plus each external provider name and URL. It does not page,
query the ledger, normalize or endorse market values, or create a cache.

This is timestamped off-chain analytics with `certified: false` and
`point_in_time_guaranteed: false`. A valid ledger principal does not guarantee
that the Dashboard service indexes that ledger. Use `icq icrc ledger token`
for current supply reported directly by `icrc1_total_supply`; the two reports
are not silently reconciled. See
[0.27 Bounded Official ICRC Analytics](design/0.27/0.27-design.md) for the
authority and series/count bounds contract, and
[0.37 ICRC Account and Holder Index Reporting](design/0.37/0.37-design.md) for
cursor-page and exact-detail behavior.

Ledger-wide live reports:

```bash
icq icrc ledger capabilities mxzaz-hqaaa-aaaar-qaada-cai
icq icrc ledger token ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc ledger index mxzaz-hqaaa-aaaar-qaada-cai
icq icrc ledger transactions ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc ledger block-types ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc ledger archives ryjl3-tyaaa-aaaaa-aaaba-cai
icq icrc ledger tip-certificate mxzaz-hqaaa-aaaar-qaada-cai
```

Transaction queries can follow ledger-supplied archive callbacks explicitly.
Following uses each supplied callback method name with the ICRC-3 argument
and result contract; callbacks execute as queries. Ledger blocks and archive
ranges must fit the requested page and reported log length without overlapping.
Following is one hop,
with at most 100 callbacks and 64 MiB of cumulative Candid replies admitted
for decoding; an over-budget reply is discarded before decoding. Each replica
response retains its existing 8 MiB transport cap. Archive replies
must contain unique block ids inside their callback ranges. Callback failures
and exhausted byte budgets are reported separately from returned blocks.
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
Both live operations require canonical newest-first transaction ids below the
exclusive requested cursor and at or above the index's oldest known id. Invalid
pages fail without changing accepted refresh progress or the complete snapshot.
Sparse ids, arbitrary-size ids, and valid empty pages remain supported. Raw
ICRC numeric fields use plain decimal strings without display separators.

The ICP ledger does not export ICRC-106 index discovery. Supply its official
index explicitly when querying ICP account history:

```bash
icq icrc account transaction page \
  ryjl3-tyaaa-aaaaa-aaaba-cai aaaaa-aa \
  --index-canister-id qhbym-qaaaa-aaaaa-aaafq-cai
```

## Cache root and recovery

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-query/ic-query-cache-lifecycle.svg" alt="IC Query cache lifecycle showing cache hits, refresh and validation, atomic replacement, and preservation of the previous snapshot after failure">
</p>

The CLI selects one user-level cache root:

1. `ICQ_CACHE_ROOT` when set to an absolute path;
2. `$XDG_CACHE_HOME/ic-query` when `XDG_CACHE_HOME` is nonempty and absolute; or
3. `$HOME/.cache/ic-query`, with an absolute `HOME`.

Empty values are ignored. A relative `ICQ_CACHE_ROOT` is an error; a relative
`XDG_CACHE_HOME` is ignored in favor of `HOME`.
It never discovers or migrates repository-local `.icq` directories.

Managed cache IO is capability-rooted beneath the selected cache root. On Unix,
managed directories and files must deny group and other write access. Readable
directories such as `0755` and regular files such as `0644` are supported without
permission repair. New directories use `0700`; new files use `0600`.
Symlinks, path escapes, nonregular files, and unsafe modes fail as
storage-authority errors and do not trigger automatic
invalid-content repair. Explicit output paths supplied by a caller are not
managed cache paths, but a refresh export must not alias its managed snapshot
or refresh lock, even during a dry run. Subnet Catalog exports also protect
Registry history and its writer lock. Group/world-writable paths remain errors
and require explicit operator correction.

`icq cache status` inventories known complete snapshots across this root. Each
row keeps generic header integrity, age state, file size, applicable stale
threshold, invalid-content recovery policy, and inspection errors separate.
The report sets `family_validation_performed` to `false`: readable generic
headers are not claims that an owning family's semantic validator would accept
the complete payload. It also reports `active`, `available` and `invalid`
refresh coordination files. Status briefly probes existing kernel locks without
writing file contents; file presence and age do not establish ownership.
Coordination files remain empty and persistent after refresh completion.

The scan is local-only, bounded to 10,000 cache and lock candidates, rejects
symlinks, skips refresh-attempt sidecars as report rows, and never refreshes,
repairs, removes, or probes process liveness. It still validates the authority
of every traversed cache-root entry. Large unmanaged histories stop at their
leading header/completeness boundary instead of scanning complete row arrays.
Small age-managed files are fully JSON-parsed for syntax before receiving
`fresh` or `stale`, but family-specific loaders remain authoritative for
identity and semantic validation.

Complete snapshot refreshes use a lock. Paged proposal, neuron, and
account-history collections also use a separate attempt sidecar so a failed or
capped refresh can retain available progress while leaving the previous
complete snapshot unchanged. The exact-version joined topology cache uses its
lock and atomic replacement without an attempt sidecar. Stale or malformed
locks are reported but are not automatically deleted; remove a lock only after
confirming that no refresh is active.

Live Subnet Catalog loads and refreshes reuse endpoint-isolated Registry
history across CLI processes at
`<cache-root>/nns/ic/subnet-catalog/history.json`, with a separate
`history.lock`. This bounded schema-1 transcript avoids repeating validated
history pages; every live acquisition still reads freshly pinned catalog
records. It neither raises assurance nor changes catalog freshness. Ordinary
cache hits and cache-only reads do no history IO, and convenience dry runs
use memory-only history.

`icq cache status` lists the transcript and writer lock as
`nns/registry-history`, with unmanaged age and explicit recovery. Invalid
transcripts can be replaced during an already-authorized live acquisition;
filesystem authority and IO failures remain errors. See the
[history reuse contract](design/subnet-catalog-acquisition-performance.md#cross-process-transcript-contract)
for retention bounds, concurrency, and progress.

Cache identity describes collected evidence, not its presentation. Sort,
limit, verbosity, and output format do not create alternate complete
snapshots. See [Cache Policy](design/cache-policy.md) for the full contract.
