# Two-sample subnet activity

Canic needs bounded evidence of replicated-message activity when inspecting a
subnet during maintenance. `icq topology subnet activity SUBNET_ID` takes exactly
two successful observations through the existing certified metrics reader. It
waits ten seconds after the first read by default; `--interval-seconds` selects
1–60 seconds. Invalid intervals fail before any network call. No scheduler,
persistent history, background polling or automatic retry is introduced.

The schema-1 report retains both complete metrics observations and the requested
wait in seconds. Each read retains its own source, local completion time and
verification evidence. The requested wait is neither the actual total runtime nor
a certified-state time interval: each read has its existing 20-second asynchronous
deadline, and the second read starts after the wait. Synchronous Agent work and
runtime scheduling retain the limitations in the [metrics contract](subnet-metrics.md).

The report classifies the second replicated-message counter as increased (with
the exact unsigned difference), unchanged or decreased. Comparison follows read
order, not local wall-clock ordering. Preserve raw values even if the wall clock
moves backwards. The [IC state-tree contract](https://docs.internetcomputer.org/references/ic-interface-spec/#subnet-information)
defines this as a cumulative counter. A lower second observation is anomalous or
out of order; do not wrap subtraction, invent a reset time or emit a negative
throughput. No rates or cycle deltas are computed.

Two reads may return repeated or differently aged certified state under the
existing 300-second certificate-age policy. Their local completion times do not
prove the certified sample times, continuous activity or a throughput interval.
An increase only establishes that the observed totals differ. Equal totals do
not prove downtime, and decreases do not identify a maintenance cause. The
report supplies no uptime percentage, complete observation window, spare
capacity or health verdict.

On first-read failure, return its typed cause without a second request. On
second-read failure, retain the first observation and typed cause in the library
error, but return no successful comparison. The CLI keeps its established
failure contract: nonzero exit, diagnostic on stderr, no success JSON on stdout.
Diagnostic text must distinguish first and second failures without claiming
subnet downtime. Cancelling the operation schedules no further work.

The library owns the two-read policy and comparison; the CLI owns argument
parsing and pure presentation. Reuse the metrics reader and its transport/codec
owners. A private injectable reader permits deterministic scheduling/failure
tests without introducing a public source trait or second transport abstraction.

## Predecessor lessons and qualification

Reviewed legacy revision
[`5b4951b`](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283),
its [metric request contract](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic/model/requests/metrics.rs),
[live reader and tests](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic/live/metric)
and [API consolidation history](https://github.com/dragginzgame/ic-query-legacy/commit/b1fca76345efb465d327984d70e9862e1127bbe9).
The predecessor made collection windows explicit and retained raw source values.
Carry those invariants into interval and exact-value regressions. Its REST
time-series queries and broad metric catalog do not establish certification or
fit this two-read operation; no legacy source is copied.

Qualification covers interval bounds before IO, exactly two reads separated by
the requested wait, first/second failure without retries, retained first-sample
evidence, equal/decreasing/full-width totals, wall-clock reversal, raw JSON and
CLI argument/help/presentation contracts. Use Tokio's paused clock for scheduling
tests. Live smoke evidence is separate from offline fixtures and does not prove
continuous availability or native macOS behavior.

On 2026-11-14, the Linux smoke check against `https://ic0.app` for subnet
`tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe` returned
verified metrics at local Unix times `1794661739` and `1794661749`, with the
requested ten-second wait. Replicated-message totals were `6904392758` and
`6904393451`, producing `{"kind":"increased","by":693}`. Both observations
retained their own evidence; this difference is not a measured throughput rate.
Six activity tests, five presentation tests and nine CLI process tests passed,
as did strict Clippy, Rustdoc, formatting and local document links.
