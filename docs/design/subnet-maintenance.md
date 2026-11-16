# Bounded subnet maintenance evidence

Status: implemented with the existing published Platform-backed readers.
Existing configuration, changes and activity commands retain their contracts.

## Decision and evidence boundary

Canic needs to inspect one subnet during planned maintenance: what registered
configuration changed, whether its requested version is registered, and whether
two observed replicated-message totals differ. Query supplies those facts; Canic
decides whether to continue, investigate or perform the next maintenance action.
There is no automatic healthy/ready/completed verdict.

Use the existing published Platform-backed configuration and certified metrics
readers. No new supplier capability is required for this slice. Running replica
version, continuous uptime, execution of halt/resume, node readiness and completed
maintenance remain explicitly unobserved. Node Rewards is not a substitute source.

The surface is `subnet::maintenance::fetch` and
`icq topology subnet maintenance SUBNET_ID`, with `--interval-seconds` (1–60,
default 10), optional `--expected-version` and `--json`. The expected version is
an exact raw Registry string of at most 256 UTF-8 bytes; explicit empty is valid
and distinct from no expectation. Matching it only means the configured string
matches. Do not resolve empty cloud-engine versions or claim running-version
agreement. Membership/halt targets and an application maintenance plan remain
Canic inputs outside this first report.

## Acquisition

Use a single coordinated sequence for one subnet:

1. Read configuration, then certified metrics.
2. Wait the requested interval after both first-round attempts finish.
3. Read configuration, then certified metrics again.

Do not compose the existing two independent live samplers: each owns a wait and
would produce differently aligned acquisition windows. Reuse their leaf readers
and comparison semantics instead. Extract only the small activity comparison
needed by both callers; do not add a generic sampler or public transport trait.

Validate interval and expected-version bytes before IO. Each leaf retains its
20-second deadline, 1 MiB HTTP ceiling, fixed mainnet root/endpoint and existing
field/codec limits. At most four logical reads, one wait, no retries, background
tasks, node fanout, persistence or pagination. Sequential asynchronous bounds
total 80 seconds plus the requested wait (at most 140 seconds), plus bounded
synchronous work and runtime scheduling. Cancellation drops pending work and
starts no further read.

The first configuration read anchors the report; if it fails, stop without a
report. After that, a failed metrics or second-configuration read is retained and
does not prevent the remaining scheduled independent evidence reads. Never retry
a failed observation or replace it with zero/empty state. A failed first metric
does not make the second metric useless: retain it even though activity comparison
will be unavailable.

## Schema and interpretation

Schema 1 retains the selected subnet, requested interval/optional expectation,
local start time and monotonic elapsed duration. Retain every acquired input with
its original source, assurance and timing evidence, plus typed failed-read
outcomes. JSON projects stable error categories and relevant bounded admission
facts, while Rust retains original typed causes. Raw diagnostic text is not a
classification mechanism or terminal content.

Configuration comparison reuses `subnet::changes::compare`: sorted members
added/removed and differences in raw version, subnet type and both halt flags.
Preserve its identity, schema, membership, mutation-version regression and
same-version contradiction refusals. Retain both inputs on refusal; the metric
comparison can still be available. Do not duplicate the retained configurations
inside both the acquisition and comparison sections: give each observation one
canonical location and reference it from derived results.

Activity comparison retains exact increased/unchanged/decreased semantics from
`subnet::activity`. Compute a delta only for an increase. A decrease is not a
diagnosed reset, equal totals do not prove downtime, and differing totals do not
establish current continuous progress. Preserve both metrics observations and
their u128 cycle values as existing decimal strings. No rate or uptime percentage.

Expected-version assessment is independently available for each successful
configuration: `not_requested`, `matches` or `differs`. A missing second read
makes its assessment unavailable, not different. Halt flags remain separately
labelled configuration values; activity does not override or validate them.

Keep `atomic_snapshot = false`. Registry query signatures do not certify result
consensus. Metrics certificates do not certify the unrelated Registry reads.
Record mutation times, local observation times and the requested wait do not
establish a shared certified interval or Registry version. Do not order evidence
by local wall clocks; they may move backwards. Matching selected fields cannot
exclude intermediate changes.

`collection_complete` requires all four reads to succeed. Comparison availability
is separate: a configuration comparison may be refused despite four successful
reads. CLI exit 0 requires complete collection and available comparisons; partial
or refused-comparison evidence is emitted on stdout with exit 1. A version
mismatch, configuration change, unchanged/decreased activity, or halt flag is a
successful evidence outcome rather than a process error. Syntax errors exit 2.
Existing commands keep their existing failure/output contracts.

## Presentation and regression acceptance

The CLI owns one compact bordered table using `ic-host-text`: before/after
configured version, raw registered subnet type and halt flags, membership counts/differences, message totals
and optional expectation matches. Show unavailable cells explicitly. Follow with
one coverage/source summary and the unobserved running-version/uptime/completion
boundary. JSON retains raw evidence and exact values. Runnable subnet examples
are in the root and CLI changelogs.

Required focused regressions:

- Invalid intervals/expectation bytes before IO; explicit empty expectation;
  exact comparison without case folding, trimming or normalization.
- Exact four-read order and one wait after both initial attempts; first-anchor
  failure stops; later failures preserve evidence and do not trigger retries.
- Cancellation during each read/wait; four independent deadline outcomes.
- Configuration refusal invariants, retained inputs and independent metrics;
  first/second metric failures distinguished from valid zero counters.
- Equal/increasing/decreasing/full-width counters, backwards local clocks,
  missing versus observed-empty membership, expected-version disagreement.
- Partial JSON/exit 1, complete disagreement/exit 0, source flags, terminal
  escaping and bounded text output. Existing changes/activity regressions must
  continue passing after any shared comparison extraction.

## Qualification

Local Linux verification on 2026-11-15 passed the maintenance acquisition and
cancellation tests, existing activity/configuration comparison regressions, CLI
unit/process tests, Clippy and topology rustdoc. Live one-second observations of
the NNS subnet succeeded in text and JSON; an explicit empty expected version
produced `differs` with complete evidence and exit 0. These live checks do not
establish uptime. The subsequent [hosted CI run](https://github.com/dragginzgame/ic-query/actions/runs/43704031732)
passed offline Clippy, rustdoc and tests on Linux x86-64 and macOS ARM64/x86-64
for delivery commit `263a8e92da873543825c25e7dd6530c4db5ab673`.

The focused review of the prior delivery and this increment found no actionable
drift in the touched acquisition and presentation paths. The new report reuses
leaf readers, the configuration comparator and message-counter classification;
it adds no transport, retry framework or persistent history.

## Legacy guidance

Reviewed frozen legacy `5b4951b23c2adfcd3c615961f131e766c4b50283`:
the [metric window contract](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic/model/requests/metrics.rs),
[exact-version topology design](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/docs/design/nns-subnet-topology.md)
and [authority regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/subnet_catalog/tests/authority.rs),
plus [API consolidation history](https://github.com/dragginzgame/ic-query-legacy/commit/b1fca76345efb465d327984d70e9862e1127bbe9).
Retain explicit windows, raw values, source identity and separate assurance.
Legacy REST time series, cache freshness and global version-pinned catalogs do
not make these independent live reads atomic or prove maintenance completion.
No legacy source, cache or generic metric catalog is imported.

## State-time source assessment

The independent [progress report](subnet-progress.md) now compares two authenticated
state times using the Git-pinned replacement Platform projection. It adds no
reads to maintenance. Generic endpoint status is not selected-subnet evidence;
management subnet_info remains canister-only, as documented in the
[IC interface](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/#ic-method-subnet_info).

Legacy node-status source admission, validation and live fixtures at frozen
`5b4951b23c2adfcd3c615961f131e766c4b50283` were reviewed in
[ic/node_status](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic/node_status).
They preserve raw labels, reject false certification/atomicity claims and separate
default-mainnet coverage from cloud-engine inclusion. The progress report retains
those authority lessons while selecting authenticated state rather than Dashboard
labels. No REST transport, global snapshot or cache was imported.
