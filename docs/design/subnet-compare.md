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
domains, or claim disjoint placement from incomplete records. The first slice
does not add cross-subnet relationship joins: the existing per-side concentration
evidence is sufficient for this comparison. Runnable examples are in the root and
CLI changelogs.

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

Local Linux verification on 2026-11-15 passed comparison acquisition/order,
all 15 component-failure combinations, cancellation at each acquisition and the
existing maintenance regressions. CLI unit/process tests, Clippy, rustdoc,
formatting and document-link checks passed. Live text and JSON comparisons of
the NNS and `brlsh` subnets completed with both distributions and metrics present,
exit 0 and differing configuration/resource values. Both runs took about 13
seconds; this observation does not narrow the documented worst-case budget.
macOS execution has not been qualified for this increment.

The focused drift review found no new transport, global inventory, duplicated
configuration collection or cross-report assurance claim. Acquisition stays in
the library and human projection stays in the CLI. The shared metrics outcome
now has one owner, and existing maintenance behavior remains covered.
