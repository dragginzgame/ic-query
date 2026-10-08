# Query complexity and Host reuse audit — 2026-10-08, run 01

## Scope, identities and verdict

Trigger: the maintainer requested the latest Host tooling check and a repository
audit for opportunities to reduce code complexity. This run inspected source,
registry observations and existing CI/evidence. It made no source repair,
dependency update, commit, release or deployment. The report and its artifacts
are the only local files written by this audit.

**Structural verdict: PASS WITH FINDINGS.** Two bounded simplifications are
supported by source traces. No new structural contract failure was demonstrated.
This is not a cache correctness, performance, security, live-network or release
readiness verdict. A separately owned, known release-integrity failure remains
applicable to the selected runner and is recorded below rather than hidden by
the structural verdict.

Primary method: `audits/complexity-and-technical-debt.md`, with the common
`audits/README.md` evidence contract and flow-convergence tracing. These files
are selected from Shared Tooling revision
`2687f26317952c43c685f7f799ed09288dc10a67` (0.1.22). The engineering baseline
remains independently pinned at `5a65686d1da9e039b22b3e17694ea083d82cfa9e`.
The local overlay is `AGENTS.md`; its exact bytes and the selected methods are
recorded in [source-sha256.txt](artifacts/source-sha256.txt).

Query source: HEAD `08012b4bedd288b5882c0d8134ea94aa739228f6`, package 0.48.1,
with a pending 0.48.2 ledger and pre-existing dirty dependency, shared-tooling,
release-adapter and smoke-receipt changes. See
[initial status](artifacts/worktree-before.txt) and
[status at report preparation](artifacts/worktree-at-report.txt).
The lockfile changed externally during inspection from Host 0.5.0 to 0.5.1;
the final hashed selection is 0.5.1 for artifacts/fs/tools, with no process
dependency. The audit did not apply or revert that update. Draft notes still
describe 0.5.0; reconcile those during the separately owned active release slice.

No adopted local complexity-method overlay or prior qualified comparable report
was found. This first run uses a provisional scope grounded in AGENTS.md,
README.md, the release ledger and relevant design/cache policy. Historical
complexity comparisons are N/A; file sizes were discovery aids, not scores.

Included: all four workspace crate manifests and source inventory; focused
traces through CLI NNS leaf requests, library cache/read/write/attempt ownership,
NNS archive replay, SNS/ICRC refresh consumers, output/progress ownership and
Governance process tooling. Product authorities retained are explicit network
identity, complete-snapshot cache identity, schema 1, capability confinement,
bounded IO, typed library progress and CLI-owned process output.

Excluded from a correctness conclusion: generated Candid/wire definitions,
cryptographic algorithms, all individual report semantics, network behavior and
measured performance. Their source size alone provides no cleanup justification;
they require owning domain proof if changed. No missing required native execution
is relabeled as an exclusion or PASS.

## Latest Host observation

All four official sparse-registry entries identify **0.5.1**, non-yanked, with
Rust 1.88.0 declared. [Registry evidence](artifacts/registry-observation.json)
records versions and checksums. The reviewed Host release source is
`81f9809861159def2fd0987fcb7961cda4afd969`; publication is checked independently
of CI. Its [exact-source CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37750135927)
passes Rust 1.88, Linux x86-64 and native macOS 15 Intel/Apple Silicon; the
[observed job results](artifacts/host-release-ci.json) are retained.

The committed 0.5.0 → 0.5.1 `crates/` diff is
[empty](artifacts/host-050-to-051-crates.diff). The patch adopts Shared Tooling
0.1.23's release identity checks; it does not add an API that deletes more Query
code. Host's worktree gained unrelated audit/status changes during inspection,
recorded in [host-worktree.txt](artifacts/host-worktree.txt); API comparisons use
committed release revisions, not those dirty files.

## State and decision ownership map

| Behavior and admitted choices | Owner → contract → projections | Interactions and invalid boundary |
| --- | --- | --- |
| NNS leaf list/info/refresh; data center/node operator/node provider | Inventory request constructors → common concrete requests → private leaf runner and family report adapters | Source, network, output, dry-run and refresh destination remain existing options. Four generic request selections currently admit only one actual implementation each. |
| Refresh attempts: running/complete/failed | Shared attempt validation → family attempt records → cache status/report DTOs | Failed requires a nonblank error; running/complete forbid one. Network/endpoint, progress, cursor and timestamps retain separate validation. String storage admits values outside the owned status set before validation. |
| Cache access/publication | Capability-confined Query root → shared bounded readers/writers and descriptor publication → domain error/report projection | Per-file and aggregate budgets, symlink refusal and permissions are independent safety constraints. Generic path IO cannot replace the confined root. |
| Snapshot refresh paging | Shared lock/attempt lifecycle → family page/collection contracts → complete-cache publication | Family cursor admission, exhaustion and source authority differ. Shared mechanics already converge; these policy differences are deliberate. |
| Governance child execution | Python Popen/communicate lifecycle → captured/inherited diagnostics and receipt → network cleanup | Success may hand background runtime lifetime to later cleanup; failure/timeout terminates the process group and reaps. Host OwnedChild's always-owned group cleanup does not supply that handoff. |

The CLI owns parsing, progress and output sinks. Library report builders own
source and cache semantics. The pure/core crate boundaries retain narrower
dependencies; current Host dependencies are not grounds to pull process tooling
into those graphs. Archive authentication/object manifests retain a distinct
owner from single-file atomic publication.

## Prioritized structural findings

### C1 — LOW: concrete leaf requests behind redundant generic adapters

**Evidence:** `crates/ic-query-cli/src/nns/leaf/model.rs:35–142` defines four
private traits, each with one implementation. `NnsLeafReports` then carries four
request associated types. `leaf/run.rs` constructs through those adapters, and
`nns/macros/reports.rs` propagates four selections. The three macro callers in
`data_center/reports.rs`, `node_operator/reports.rs` and `node_provider/reports.rs`
all select the same Inventory types. Repository-wide reference search found no
other implementation or selection.

**Consequence:** a common request change propagates through unnecessary private
traits, associated types and macro parameters. This is limited current
maintenance friction, not a runtime bug or a claim about generic dispatch cost.

**Simpler owner:** use the existing concrete Inventory requests in the private
runner and report-trait signatures. Delete the four constructor traits and
request-type macro selections. Keep the shared runner and genuinely different
reports, errors and renderers. This is a patch-compatible internal cleanup.

**No-build/state delta:** no framework, option or public state is needed; reuse
the existing request authority and remove unrealized implementation alternatives.
Disposition: **fix when touched**, or fix within separately authorized cleanup.
Query owns the active follow-up: [#21](https://github.com/dragginzgame/ic-query/issues/21).

### C2 — LOW: typed status is converted to a string and rediscovered

**Evidence:** public `SnapshotRefreshAttempt<Metadata>.status` is a String at
`crates/ic-query/src/snapshot_cache/attempt.rs:80`. NNS Governance, SNS and ICRC
writers receive `CacheRefreshAttemptStatus`, allocate its string label, and the
shared validator reparses it. Report projection then receives the validated enum
separately. The enum in `cache/model.rs` owns all three labels but currently
derives Serialize only.

**Consequence:** the persisted model and status projections carry two
representations of one decision. Rust callers can construct unsupported labels;
current validation rejects them, so this is not demonstrated corrupt acceptance.

**Simpler owner:** type the attempt field with the existing enum and add strict
deserialization. Remove conversions and redundant projection parameters where
the field carries the decision. Preserve cross-field, identity, field-admission,
timestamp/progress and budget validation; typing the status does not prove those.

**No-build/state delta:** narrow the representation to the existing supported
states; add no status, schema or configuration axis. **Requires a minor release**
because the public Rust field contract changes. Keep schema 1 and existing JSON
labels, hard-cut callers together, and review malformed-label error classification
at the deserialization boundary. Disposition: **accept until the next minor
public attempt-model change**. Query owns
[#22](https://github.com/dragginzgame/ic-query/issues/22); this is not 0.48.2 work.

## Three source-only change rehearsals

| Rehearsal | Expected owner and affected boundaries | Present blocker and smaller outcome |
| --- | --- | --- |
| Add a shared Inventory request field | Library Inventory model; CLI leaf construction and the three report adapters | Four private adapter traits/associated selections add propagation without a second implementation. C1 removes that plumbing while retaining report differences. |
| Carry a validated attempt status into cache reports | Shared attempt model/validator; Governance, SNS and ICRC writers/readers and public callers | Public String contract plus separately carried enum. C2 converges representations at a coordinated minor release; it preserves all other validation. |
| Replace Governance process cleanup with Host | Host process owner; Query smoke runner/receipt/network lifecycle | Successful background handoff is not the same lifecycle as owned foreground execution. Extending the shared contract may permit deletion later; a local protocol bridge presently increases maintenance and fails that contract. |

These rehearsals are not new feature or implementation requests. Proposed repair
checks for C1 are focused CLI tests for the three leaf families and focused
Clippy; C2 needs shared attempt and three family JSON/status/error projection
tests. Prepare the selected lockfile cache explicitly and keep those checks
locked/offline. No such repair checks were run by this audit.

## Known separate release-integrity failure

Query still selects Shared Tooling 0.1.22's direct runner. Its checksum
`c6409aed359942eccb58534d69adc136a08e6018c26d3035247380494cc665e3`
matches the old Host runner against which a retained command-substitute fixture
accepted a final-check tag retarget, reached its push stub and declared completion.
See [failure](artifacts/reused-release-fixture-failure.log),
[events](artifacts/reused-release-fixture-events.log) and
[output](artifacts/reused-release-fixture-output.log). Original evidence remains
at `/tmp/ic-host-051.cdcrRj/before.log` and
`/tmp/release-runner-test.Fm5yQL/push-check-mutation-tag/`.

Source inspection confirms assertions before `release-push-check` without the
post-hook checks, and incomplete completed-direct identity reconciliation.
**The final-check integrity obligation fails in the reused identical-runner
fixture.** This is serious release-integrity evidence, not a new Query execution
or a claim that a real release was compromised. No real tag, commit or push ran
in that substituted fixture or this audit.

Owning fix: Shared Tooling [#58](https://github.com/dragginzgame/shared-tooling/issues/58),
released at `0ba0ad00ed94848e54ecc82629b6b7873b7284c0` (0.1.23). Consumer adoption
already has [Query #20](https://github.com/dragginzgame/ic-query/issues/20); no
duplicate issue or local release engine is needed. Prioritize that reviewed
snapshot refresh before release readiness. Retain Query's direct-only adapter,
receipts and release boundaries; run focused owning fixtures and consumer adapter
checks, then record exact committed consumer native qualification separately.
Host [#22](https://github.com/dragginzgame/ic-host-tooling/issues/22) is already
closed with its released-source native evidence; that is not Query qualification.

## Intentional retention and verification gaps

- **Process boundary: retain until Host #5 supplies a qualifying contract.**
  Host 0.5.1 does not change OwnedChild group cleanup. The withdrawn integration
  introduced roughly 300 lines and a cross-language protocol around a roughly
  33-line process helper and failed successful background handoff. Its
  [provenance](artifacts/reused-process-candidate-provenance.md) and
  [failure](artifacts/reused-process-candidate-failure.log) are reused diagnostic
  evidence, not accepted implementation. Keep the existing lifetime and receipt
  failure ownership; [Host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5#issuecomment-6055899595)
  tracks the upstream gap. Accept until shared handoff and complete caller IO
  support permit a genuinely smaller consumer flow.
- **Confined cache roots and aggregate budgets: not debt.** Capability and
  permission checks protect a boundary beyond generic Host filesystem IO.
  Descriptor publication and bounded/hash writers already use shared ownership.
- **Selective cache headers: not debt.** Header visitors avoid reading full
  payloads. Replacing them with whole-document decoding changes IO/budget/error
  contracts. Different current family schema fields are not fallback readers.
- **Family paging and archive policy: not debt.** Cursor/count admission,
  filtered-page exhaustion and authenticated archive identities have distinct
  semantic owners. Keep shared lifecycle mechanics without a larger policy engine.
- **Large modules: no size-only refactor.** Many large files include tests or
  domain validation; no measured cost supports new module or abstraction splits.

## Checks and handoff

Performed: source/reference and owner traces, committed Host crate diff, official
registry checks for all four crates, exact Host release CI inspection, issue
duplicate/state inspection and report artifact identity recording. Active build
checks found no cargo/rustc/rustfmt process at the inspection points. Two local
structural issues were filed; existing Query #20 and Host #5 remain their owners.
Recorded source checksums were verified after writing the report; report/artifact
local links and worktree/report whitespace checks pass. These are document and
identity checks, not behavioral tests.

Reused only: earlier Linux receipt/process tests
([33 passed](artifacts/reused-receipt-tests.log)) and the two failed substitute
integration fixtures above. The receipt run used the prior Host 0.5.0 selection;
it does not qualify the externally updated 0.5.1 lockfile. Original artifacts
were preserved and copies are explicitly marked reused.

Not run: Rust compilation, fresh unit tests or Clippy, live Governance smoke,
full workspace CI, release gates, release execution, or native Query macOS
qualification. Upstream native CI is supported evidence for Host's release only.
The dirty Query worktree has no native CI result of its own. Focused repair and
adoption proof remain work for their owning issues; this report is immutable
evidence rather than a second active task ledger.
