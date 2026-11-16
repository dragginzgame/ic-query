# Compare two selected placement candidates

Status: implemented using existing published Platform-backed readers.

## Canic decision

Canic needs to compare two explicitly selected subnets before placing or
consolidating an application's workload. Present their registered configuration,
provider/location concentration and current resource observations side by side.
Canic supplies workload requirements and decides suitability. Query does not
rank candidates, recommend a merge, estimate spare capacity or execute changes.

This is a comparison across identities, distinct from `subnet changes` and
`subnet maintenance`, which compare observations of one identity over time.
Surface: `subnet::compare::fetch(left, right)` and
`icq topology subnet compare LEFT RIGHT [--json]`. Preserve argument order;
reject identical canonical principals before IO. Do not add subnet discovery,
arbitrary lists, repeated sampling or automatic candidate selection.

## Existing sources and bounds

Use the existing [distribution](subnet-distribution.md) and
[metrics](subnet-metrics.md) readers. Distribution already owns its admitted
configuration: do not fetch or retain a duplicate configuration report.
Published Platform capabilities suffice; no new dependency or protocol codec
is required.

Acquire left distribution, left metrics, right distribution, right metrics in
that order. Continue independent acquisitions after any failure. Each
distribution retains its 60-second deadline, 128-member admission and concurrency
of four; each metrics read retains its 20-second deadline. Total asynchronous
budget is at most 160 seconds, plus bounded synchronous work and scheduling.
There are at most 774 logical source calls (two times 386 distribution queries
plus two metrics reads). Existing byte and record limits remain unchanged.
No retries, caches, spawned tasks or background collection. Cancellation drops
pending work and starts no later acquisition.

Each distribution selects its own requested Registry version. Expose both
versions explicitly, including whether they agree when both are available.
Do not call independent collections a common-version inventory. A future
common-version collector would need its own justified contract; do not refactor
the current reader merely to hide this distinction. Certified metrics are
separate observations and have no Registry-version alignment.

## Report and failures

Schema 1 retains left/right identities, local start time, monotonic elapsed
duration and four typed outcomes. Each successful input is owned once, including
all source, timing, verification and partial-coverage evidence. Initial failures
of a component are recorded alongside the other components; they do not erase
the pair report. Rust retains typed causes, JSON retains stable error categories
and bounded admission facts without using diagnostic text as a classifier.

`collection_complete` requires both distributions to be complete and both
metrics reads to succeed. A returned partial distribution remains inspectable.
Both sides failing still produces an explicit incomplete report, not empty
success. Parameter or initial clock failure returns no report. CLI exit 0 means
complete acquisition, not preferred placement; partial reports go to stdout
with exit 1. Invalid syntax exits 2. Different versions or values are successful
evidence outcomes and do not themselves cause failure.

Use `atomic_snapshot = false`. Never convert missing data to zero, empty
membership, an absent provider or proof of downtime. Preserve observed-empty
membership distinctly. Do not compute rates, cross-subnet cumulative-counter
deltas, resource totals or a score. In particular, lifetime cycle/message totals
are not comparable workload rates; keep them in the retained raw metric inputs
without using them to rank the sides.

## Human presentation

Use `ic-host-text` bordered tables with left/right columns and an identity legend.
Show raw registered type, configured version, halt flags, member count, canister
count and state bytes. Show provider and location coverage independently, with
known/unknown counts and the existing concentration summaries. Unavailable rows
stay visible. Keep every source's version/time/assurance available in JSON and a
compact text evidence section.

Do not interpret shared provider or location labels as verified common failure
domains, or claim disjoint placement from incomplete records. Runnable examples
are in the root and CLI changelogs.

## Observed infrastructure overlap

`infrastructure_overlap()` derives provider and data-center intersections from
the two retained distribution summaries, without more reads or stored state.
JSON adds an `infrastructure_overlap` field; existing report fields, Rust
construction and schema 1 remain unchanged. Either missing distribution produces
null, distinct from two acquired distributions with empty intersections.

Each relationship kind retains shared canonical identities in sorted order, node
counts separately for each side, each side's unknown-node count and an independent
`coverage_complete` flag. Provider coverage can be complete while location
coverage is partial. A metrics failure does not prevent this comparison.
Observed intersections remain available under partial coverage or different
Registry versions; the original versions and times still qualify their meaning.
No identity is labelled left-only/right-only, and no absence becomes a disjointness
verdict. Counts never imply simultaneous assignment or verified failure domains.

Use a linear merge of the existing canonical groups (at most 128 per side/kind).
Do not duplicate relation resolution, add another inventory or compare raw
region/owner labels. Data-center groups use successfully read location records,
not merely operator references to an unavailable record. The derived API accepts
the admitted reports produced by `fetch`; it does not authenticate or validate
arbitrary caller-built structs. Keep CLI overlap counts and unknown coverage in
a bordered summary; raw shared identities and node counts remain in JSON.
The optional text-only `--overlap-details` adds shared-identity tables in canonical
order, with per-side node counts and terminal escaping. It uses the already
derived overlap (at most 128 rows per relationship kind), performs no extra reads
and leaves exit policy unchanged. It conflicts with `--json`; JSON already
retains every shared identity. Empty tables say none observed, and a missing
distribution still makes overlap unavailable.
The text summary also sums each side's observed node assignments to shared
identities and displays them over that side's full selected membership, including
unresolved nodes. These exact fractions are display projections of retained counts,
not probabilities or estimates of total overlap. Do not use resolved-only counts
as denominators, combine provider and location fractions, or hide unknown coverage.

## Regression acceptance and predecessor guidance

Test duplicate selection before IO; exact acquisition order; every component
failure independently; partial distribution versus failure; both sides failing;
cancellation at each acquisition; distinct Registry versions; unknown type and
empty configured version; zero versus unavailable metrics; full-width quantities;
independent provider/location coverage; partial JSON exit 1 and complete differing
values exit 0. Verify terminal escaping without changing raw JSON.

Inspected frozen legacy revision `5b4951b23c2adfcd3c615961f131e766c4b50283`:

- [Exact-version design](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/docs/design/nns-subnet-topology.md)
  separates canonical topology from diagnostics and preserves missing relations.
- [Version projection](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/nns/topology/report/registry_versions.rs)
  retains each component's provenance instead of inventing shared authority.
- [Relationship implementation](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/nns/topology/report/relations.rs)
  keeps unknown references explicit; reuse that invariant, not its global index.
- [Authority regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/subnet_catalog/tests/authority.rs)
  demonstrate why incomplete evidence cannot support stronger assurance claims.
- [API consolidation history](https://github.com/dragginzgame/ic-query-legacy/commit/b1fca76345efb465d327984d70e9862e1127bbe9)
  informs reuse of existing report boundaries rather than a second query stack.

The old global catalog, governance enrichment and persistent cache are not
needed for two selected candidates. No legacy code or state is imported.
The implementation retains the current reader bounds. Metrics acquisition outcomes
are owned by the metrics module and reused by maintenance and comparison;
maintenance retains its existing type path as a re-export and its JSON shape.

## Qualification

The subsequent overlap extension passed focused Linux comparison/CLI tests,
Clippy, rustdoc, formatting and document-link checks on 2026-11-15. A live JSON
comparison of the same NNS/`brlsh` pair returned complete provider/location
coverage, seven shared providers and five shared data centers. These are observed
declarations, not an independence or simultaneity claim. The extension has not
yet received hosted macOS qualification. Drift review confirmed no extra reads,
supplier dependency changes, duplicate retained overlap state or changed exit policy.

Local Linux verification on 2026-11-15 passed comparison acquisition/order,
all 15 component-failure combinations, cancellation at each acquisition and the
existing maintenance regressions. CLI unit/process tests, Clippy, rustdoc,
formatting and document-link checks passed. Live text and JSON comparisons of
the NNS and `brlsh` subnets completed with both distributions and metrics present,
exit 0 and differing configuration/resource values. Both runs took about 13
seconds; this observation does not narrow the documented worst-case budget.
The subsequent [hosted CI run](https://github.com/dragginzgame/ic-query/actions/runs/43704031732)
passed offline Clippy, rustdoc and tests on Linux x86-64 and macOS ARM64/x86-64
for delivery commit `263a8e92da873543825c25e7dd6530c4db5ab673`. Hosted checks
do not establish live-network behavior on those hosts.

The focused drift review found no new transport, global inventory, duplicated
configuration collection or cross-report assurance claim. Acquisition stays in
the library and human projection stays in the CLI. The shared metrics outcome
now has one owner, and existing maintenance behavior remains covered.

## Optional registered settings

`fetch_with_settings(left, right)` and CLI `compare --settings` compose the
existing comparison followed by left and right settings reads. The original
`fetch` API, constructors, JSON and default CLI reads remain unchanged. The opt-in
schema-1 wrapper owns the original `comparison` once, plus `left_settings` and
`right_settings`, total elapsed duration and overall `collection_complete`.
No settings declaration is aligned to the distribution's Registry version;
retain each settings record's mutation version, timestamp and signature evidence.

The acquisition order is the original four components, then two independent
settings reads. Even if the comparison is partial or the first settings read
fails, attempt the remaining settings read. Duplicate selection or failure to
establish initial comparison time stops before settings. Total bounds are 200
asynchronous seconds and 776 logical source calls, with unchanged leaf limits.
Cancellation drops the pending acquisition without starting later work.

Each settings outcome retains its report or typed error; JSON failures use stable
categories and Registry rejection codes, never unescaped diagnostics. Overall
completion requires the original comparison and both settings reads to succeed.
Partial output exits 1; differences, absent declarations and explicit false/zero
remain valid evidence. Text adds a bordered left/right settings table with units
and independently retained evidence. No inferred defaults, headroom subtraction,
TEE readiness, requirement matching or candidate score is introduced.

Regression coverage must retain default comparison behavior and verify settings
read order, continuation after failures, cancellation at both added reads,
duplicate selection before IO, raw optional values, partial JSON/exit status and
separate source versions. This composition uses the settings contract's reviewed
legacy authority/provenance lessons and adds no supplier codec or transport.

Focused Linux verification on 2026-11-15 passed comparison/settings regressions,
all CLI unit/process tests, Clippy, topology Rustdoc, formatting and document links.
The installed CLI's live NNS/`brlsh` comparison completed with both settings reads
and exit 0 in 15.67 seconds. Settings mutation versions differed from each other
and from the distributions' requested version, as retained in the output. This
does not narrow the worst-case bound or qualify native macOS behavior. Drift
review confirmed reuse of leaf readers and one shared settings row formatter,
without a generic collection framework, extra default reads or sibling changes.
