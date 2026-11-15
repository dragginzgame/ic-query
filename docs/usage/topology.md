# Using topology evidence for Canic decisions

IC Query supplies observations for Canic's planning. Canic owns application
requirements, topology changes and deployment. Use each report for the question
its source can answer; retain its evidence when comparing results.

Run the setup commands below from the `ic-query` repository root after the
Rust/tool prerequisites in [development setup](../development/setup.md):

```bash
make fetch-dependencies
make icq
```

Fetching needs network access. Installation replaces the Cargo-installed `icq`
and checks PATH. The report commands below make anonymous, read-only mainnet
requests; they require network access but no credentials or token transfers.
Reinstall after changing the checkout to use its new CLI behavior. Tabular text
uses `ic-host-text` borders and visible column delimiters; `--json` retains raw values.

## Locate an application and identify candidates

Look up the ICP ledger canister's subnet, or list registered subnet IDs:

```bash
icq topology canister show ryjl3-tyaaa-aaaaa-aaaba-cai --json
icq topology subnet list --json
```

A mapping is routing evidence, not proof of canister existence or uptime.
Registration identifies a candidate to investigate; it does not establish spare
capacity, placement eligibility or permission to deploy there. Query does not
rank candidates automatically.

## Inspect one subnet before planning maintenance

These examples use the NNS subnet so they run without replacing placeholders.
Select the actual candidate subnet from the preceding reports for application
planning; this example is not a placement recommendation.

```bash
icq topology subnet configuration tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology subnet membership tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology subnet metrics tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
icq topology subnet activity tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --interval-seconds 10 --json
```

| Planning question | Available evidence | Further input needed |
| --- | --- | --- |
| What configuration is registered? | Membership, configured replica version, raw subnet type and halt flags | Running version and maintenance completion; an empty version may use a separate cloud-engine record |
| Do registered members match the returned node keys? | Membership comparison retains both reads and their set differences | Transition history, source age and live-node observations; neither equality nor a difference is a health verdict |
| How much state and activity is observed? | Certified canister count, state bytes, nominal cycles and message total | Applicable limits, workload demand and measured headroom before sizing or consolidation |
| Did the message counter change across two reads? | Activity retains both certified samples and the direction of change | Sustained observations, failure classification and coverage before estimating uptime |
| Can workloads move to a TEE-based authentication strategy? | Selected-node Registry operator, optional chip ID and configured HostOS | Verified attestation, usable hardware and an Auth trust policy; a Registry chip ID alone will not establish these |

The membership report includes its own configuration read. When only that
comparison is needed, use it directly instead of also issuing the standalone
configuration command. Each command acquires new evidence; their results are
not an atomic snapshot.

## Compare configuration observations

```bash
icq topology subnet changes tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --interval-seconds 10
```

The second read starts ten seconds after the first completes. The wait accepts
1–60 seconds; each read has its own 20-second deadline. Text prints changed
configured fields and member IDs; `--json` retains both full observations.
No selected changes is a successful result. It cannot exclude changes between
samples or prove maintenance completion, running software or node liveness.
Library callers can also compare two previously acquired configuration reports
without IO through `subnet::changes::compare`.

## Read the registered nodes of one subnet

Collect node operator, configured HostOS and registered chip IDs for the selected
subnet's membership. Configuration is included, so no separate configuration
command is needed:

```bash
icq topology subnet nodes tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

The collection allows up to 128 members, reads in batches of four and has a
60-second overall budget including configuration. Every registered member gets
an outcome, including failed, cancelled and unstarted reads. Inspect
`all_node_reads_succeeded` and each row before comparing machine configurations.
This command prints partial evidence to stdout and exits 1 if any node read is
unavailable. Exit 0 means all selected reads succeeded, not that nodes are live.
Separate latest-record reads do not constitute an atomic snapshot.
Omit `--json` for a compact table: full node IDs, operator/HostOS references,
chip-ID presence and read status. The keys beneath the table list each distinct
operator and configured HostOS version once. JSON retains raw chip IDs and all
per-node evidence and timing.

## Compare provider and location concentration

```bash
icq topology subnet distribution tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe
icq topology subnet distribution tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe --json
```

This report resolves the global Registry version once and requests subnet,
node, operator and data-center records at that version. It includes the selected
membership, so no preceding configuration read is needed. Counts describe nodes
in that membership; a shared operator or data center is fetched only once.

Text presents provider and data-center counts with independent resolved/unknown
totals. JSON includes every node's `relationships`, each source record and read
outcome, plus raw region/owner group counts. A failed location read does not erase
a resolved provider. A known location reference is distinct from read location
details; missing or invalid references remain explicit. Oversized reference
content is refused with its byte length retained, never silently truncated.

Collection permits 128 members, batches of four and 60 seconds across all phases.
Partial results remain on stdout with exit 1; inspect `complete` and independent
coverage before comparing candidates. Initial version/configuration failure
produces no report. A common version is not consensus certification. Registered
region and owner text is not verified geography or proof of independent failure
domains, usable capacity, uptime or TEE protection.

## Inspect one node's registered configuration

Select a node ID from subnet configuration or an independently known candidate.
The node report does not require subnet assignment, perform provider/location
joins or resolve running HostOS software. This concrete example reads one public
node record:

```bash
icq topology node show nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae --json
```

JSON retains absent, empty and nonempty chip/HostOS values distinctly. A missing
chip ID is unknown registration data, not proof of missing hardware capability.
Any later authentication change requires evidence and policy beyond this report.

## Compare one node's registration

Before maintenance or placement decisions, compare one node's registration twice:

```bash
icq topology node changes nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae --interval-seconds 10
icq topology node changes nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae --interval-seconds 10 --json
```

The second read starts after the requested wait following the first completion.
Text shows changed operator, chip registration and configured HostOS fields;
JSON retains raw optional values and both observations. A chip change is visible
even when its byte count stays the same. Absent and present-empty remain distinct.
No selected changes is success, not proof of stability between observations.
This does not establish when a node came online, verified attestation, usable TEE
protection or measured software. Library callers can compare acquired reports
without IO through `node::changes::compare`.

## Inspect one existing cloud engine

```bash
icq topology cloud-engine show engine-0b56b29a-7ea0-40b8-a963-6c4539ea4414
icq topology cloud-engine show engine-0b56b29a-7ea0-40b8-a963-6c4539ea4414 --json
```

Engine IDs are opaque text, not subnet or canister principals. This public
lookup returns the subnet/operator binding, engine-record node snapshot and
embedded Registry-copy membership. The bordered table compares the two lists;
JSON retains their original order, optional values and sorted set differences.
An absent copied subnet makes comparison unavailable, rather than treating
membership as empty. Empty configured version text denotes the source's standard
engine version; it is not an observed running version. Deletion timestamps are
retained when present.

The query has a 20-second deadline and admits at most 128 nodes per list.
An unknown or anonymously hidden ID returns a successful `not_returned` report
with exit 0. Failures exit 1 without a report. No Registry-copy version or
freshness is exposed, and matching lists do not prove liveness or capacity.
The example is a public observation, not a promise that this engine remains present.

## Match candidates to public price declarations

```bash
icq topology cloud-engine candidates --max-pages 20
icq topology cloud-engine candidates --max-pages 20 --json
```

The default is ten availability pages; 20 allows up to 2,000 observed nodes. This
composes a 60-second availability collection and a 30-second price collection.
The table lists every matching public declaration with its exact daily amounts
and candidate count. Counts overlap when multiple declarations match a node.
JSON retains both reports and each node's indices into `pricing.report.prices.entries`.

Matching uses exact class/provider/data-center scope. Missing declarations are
unknown prices, not zero cost. Even one match is not a quote: precedence and
customer-specific discounts remain unobserved. No tariff is selected or total
deployment cost calculated. Price-read failures retain availability with unknown
matching coverage; fee-only failure still allows matching to supplied gross prices.
Partial collection exits 1. Exit 0 means collection succeeded, including when
nodes have no matching declaration or several; inspect coverage before planning.

## Read cloud-engine price inputs

```bash
icq topology cloud-engine prices
icq topology cloud-engine prices --json
```

The table lists exact whole cycles per node per day. Keys identify the optional
provider and data center around the class. JSON preserves those scopes, raw
class tags, signed source timestamps and exact decimal cycle strings. A missing
scope denotes a broader declaration; present-empty text remains distinct.

Gross prices already include their fee. The separate network markup is a
fraction added to net (`0.25` means +25%); it must not be added again to gross or
used to recalculate older declarations. No currency or monthly conversion is
performed. These are price inputs, not binding quotes or proof of available nodes.

The report admits at most 1,000 unique identities and spends up to 30 seconds
across both reads. If the fee fails after prices succeed, output retains prices
with `complete: false` and exit 1. Inspect `network_fee` for its typed outcome.
Exit 0 establishes successful admitted reads, not an atomic or current inventory.

## Inspect public cloud-engine candidate nodes

```bash
icq topology cloud-engine available
icq topology cloud-engine available --json
icq topology cloud-engine available --max-pages 20 --json
```

Text groups candidate nodes by raw class and data-center reference, showing
node, distinct-provider and registered-chip counts. JSON retains every admitted
node's identity, operator/provider, raw class, optional data center and source
`sev_enabled` flag, grouped into pages with request/response cursors and timing.
IP addresses are omitted. The flag records a registered chip ID, not verified
attestation, hardware readiness or permission to weaken authentication.

This requests public eligibility for a new engine only. Collection reads copy
status, then pages of up to 100 nodes, bounded by default to ten pages/1,000 nodes
and 60 seconds total. `--max-pages` accepts 1–100; the example with 20 permits
up to 2,000 nodes. The overall deadline remains 60 seconds. JSON records
`page_limit` and `node_limit`; increasing them never guarantees exhaustion.
Status is an earlier source observation; its `caught_up` flag
does not prove current Registry freshness or pin later page versions.

Inspect `pagination_exhausted` and `stop`. Exhaustion exits 0, including an empty
final result; it does not establish an atomic inventory or reserve capacity.
`point_in_time_inventory_complete` and `atomic_snapshot` remain false. Nodes may
change hands between pages even when every page reports the same Registry version.

A changed page version, repeated cursor, duplicate node, limit or failed read
stops traversal and retains previously admitted pages with exit 1. An unready
copy is distinct from an empty available pool. If the initial status query fails,
there is no report; if the first page fails, the status and failure remain in
partial output. There is no automatic restart or cross-command resume claim.

## Preserve uncertainty when consuming results

Use the typed library reports or JSON for Canic integration. Text is a
human-facing view. JSON preserves raw values, including large integers; select
a consumer that retains their precision rather than coercing all numbers into
floating-point values. Configured strings are not measured software identities.

Retain endpoint, source, record mutation metadata and verification flags with
the values. A Registry query signature is distinct from consensus certification;
a local observation timestamp is distinct from certified time. Separate reports
do not acquire a shared version merely because they were requested close together.

A failed single report exits unsuccessfully and emits no success JSON. The
`subnet nodes`, `subnet distribution`, `cloud-engine prices` and
`cloud-engine available` and `cloud-engine candidates` collections deliberately emit explicit
partial JSON with exit 1 after initial admission succeeds; initial admission
failure emits no report. Treat failures as unavailable evidence, not zero usage, empty membership
or downtime.
The library preserves typed causes; membership also retains its first report
when the second read fails. Query adds no persistent history or background retry.

Before deciding whether to add cloud engines or consolidate workloads, combine
these observations with application demand, resource limits, availability needs
and placement constraints. Query neither selects the final topology nor performs
the change. The [report contracts](../design/0.104-restart.md#report-guarantees)
define the evidence boundary for future additions.
