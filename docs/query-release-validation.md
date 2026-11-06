# Query release-group validation

Query adopts Shared Tooling 0.29.3's [group plans](release-group-validation.md)
and [roster, baseline and evidence interfaces](group-validation-adoption.md).
The existing release runner still owns source admission, metadata preparation,
Git effects and recovery. Publication order and archive admission are unchanged.

## One roster, complete product coverage

[The validation policy](../ci/release-validation.json) is the single ordered
roster for full repository CI and release plans. Full CI reads that roster once
on Linux, macOS ARM and macOS Intel. Command-line `CI_TARGETS` cannot replace it.
Shared checks that its groups match the existing Cargo release policy; Cargo's
group admission continues to own package membership and version identities.

The local selector accounts for the entire diff from the selected group's
annotated ancestor release tag. Narrow selection requires a matching qualified
host/mapping and changes confined to that group's source, tests, package README
or release notes. Unknown baselines, mixed owners, manifests, locks, features,
compiler settings, shared tooling, recipes and other paths select the full roster.
Nondefault build/compiler and tool selections also retain full validation.
Selected pin files must be contained regular checkpoint inputs; external or
redirected configuration refuses rather than escaping source custody.

| Gate | Required checks |
| --- | --- |
| Query owner-only release | Query and private Testkit Clippy and tests; Query archive; CLI/Text reverse-consumer tests and complete feature/runtime boundaries. |
| CLI/Text owner-only release | CLI/Text Clippy, tests and archives against actual Query dependencies; public API/schema integration and complete feature/runtime boundaries. |
| Shared, unknown or unqualified scope | Every stage in the full roster. |

[Scope data](../ci/release-validation-scopes.json) declares the only permitted
omissions and their reasons. Full CI runs both `query-test` (Core and private
Testkit) and `cli-test` (CLI processes, Governance harness and Text presentation).
The selected CLI plan omits `query-test` only when neither Query nor private
Testkit sources changed. Query releases always retain `cli-test`, including when
only a private Testkit fixture changes. Text changes retain the CLI processes
that consume its output. A Query/schema change during a CLI release broadens to
the full gate.

The complete feature-boundary stage remains required in both groups: portable,
host, focused adapters, generated Governance Wasm, canister public API and Text
Wasm graphs, together with public report/schema consumers. Schema admission,
dependency checks and public documentation remain required too. The roster splits
Clippy and archives into `query-*` and `cli-*` stages. Clippy uses explicit package
selections, all targets and all features; Query also retains its no-default-feature
lint. Query plans omit unchanged CLI/Text lint and archives while retaining
`cli-test` as an explicit reverse consumer. CLI/Text plans omit unchanged
Query/Testkit lint, tests and archives; Cargo still builds their Query
dependencies. CLI/Text archives use the
actual local Query and Text peers, with no package-version projection.
Full CI and every full fallback execute both groups' stages, covering all four
workspace members and all three published archives. Standalone `make clippy`,
`make check` and `make package` retain their complete workspace behavior.
Clippy already performs native compiler checks for the exact package, target,
feature and profile selections previously repeated by the standalone check
stages, with warnings denied. Query has no Clippy-specific compilation branches
or workspace build scripts. The roster therefore omits a second native
`cargo check` pass. Feature-isolated checks, Wasm checks, tests and archive
verification retain their distinct selections and remain required.
Installed Testkit/Governance and local NNS smoke checks remain in all three
native canister lanes. Source/index, publication and recovery admission remain
owned by their existing adapters.

Both groups also omit the unchanged `ci-scripts-check`,
`publish-guards-check` and `release-guards-check` automation regressions.
Repository-wide CI retains every stage. New stages default to required; unknown
exclusion targets or groups refuse rather than silently dropping an obligation.

The `candid-boundary-check` stage inventories all first-party Rust under `crates/`
once per plan, including tests, examples and the private Testkit package. Its
[declaration policy](../ci/candid-boundary.json) admits only named external IC
protocol DTOs, codec helpers and protocol fixtures. Owned reports and application
RPC models have no allowances. Missing, stale or overlapping allowances fail;
source review and codec tests still own semantic and macro-generated behavior.
See the [shared checker contract](candid-boundary.md) for its lexical limits.
Both release groups retain this lightweight stage, with no group omission.

## Evidence and recovery

Each fresh release creates a plan under `target/release-validation/run.*`, bound
to its admitted source, group, baseline, compiler, native host and build settings.
Every tracked source input is checkpointed. Shared runs the plan, then verifies
all required PASS events, timings, logs and checkpoints before emitting proof.
Excluded stages remain not applicable; failed/interrupted stages never become PASS.

Query's existing validation identity binds the release intent. The shared receipt
helper protects exact bytes of the verified plan, context, baseline, proof and
complete runner evidence. Later preparation/recovery rechecks that custody,
compiler/environment, baseline tag and original source admission. The existing
metadata owner authenticates permitted version edits; no normalized lockfile,
synthetic version projection or old-receipt conversion is introduced. Missing,
moved or changed evidence refuses. Preserve the pointer beside the release attempt
and the complete `target/release-validation/` evidence directory. Old full-gate
attempts require their original source/adapter for recovery.

## Mapping qualification

`make qualify-release-groups` runs the complete plan once, then each group's
selected plan on the same source, host, lockfile and compiler. It retains actual
commands, elapsed times and Shared proofs. This is explicit mapping qualification,
not an override for release selection; qualification proof is not a release receipt.

Native CI selects this comparison when mapping, runner, recipe, compiler or CI
inputs change, or through manual workflow dispatch. Other changes run full CI
once. Host-admission-only updates do not retrigger comparisons. All three native
hosts remain required; canister integration remains a separate unchanged lane.
Comparisons are not an unconditional extra gate on every source change.

[Host admission](../ci/release-validation-hosts.json) records only observed
host/mapping pairs. Its fingerprint covers the actual roster, recipes, selector,
runner, fixtures, compiler pin and workflow; changed mechanics invalidate admission.
Unqualified hosts keep the full release plan. Admit another host only after its
matching full/selected comparison succeeds; a Linux run does not qualify macOS.

`make release-validation-check` exercises Query's actual selector, shared runner
and receipt wiring with simulated product stages. It covers both group mappings,
full expansion, required/excluded product failures, interrupted stages, source/roster/
scope-input drift, unknown groups and evidence tampering. It also executes the
actual full/selected qualification wiring with simulated stages.
The command fixtures execute the real Make recipes with a recording Cargo
substitute. They assert exact package/feature/target selections, local archive
peers, locked/offline execution, source admission and failure propagation. Run
only those fixtures with `IC_QUERY_RELEASE_VALIDATION_CASE=commands`; they do not
compile products. The mapping fixtures separately require selected lint
and archive failures to block, and unrelated group stages to stay
excluded. Full-fallback fixtures retain every stage.

The focused UTF-8 case selects a non-ASCII omission reason through Query’s actual
selector, shared runner and retained-receipt check. Select it with
`IC_QUERY_RELEASE_VALIDATION_CASE=utf8`; the default fixture selection is unchanged.
Generic schema/path/event tests remain upstream. These fixtures establish dispatch
and custody, not product execution, native macOS support or a measured speedup.

## Current qualification status

Splitting Cargo and archive stages changes the mapping fingerprint. The new
mapping requires matched full/selected native comparisons before host admission;
all release hosts keep the full fallback until that evidence is admitted. Focused
command and mapping fixtures establish dispatch and refusal behavior, not native
qualification or a measured speedup. CI selects comparisons for this change.

## Previous native mapping admission, 2026-11-04

The earlier mapping `ae59229f106af387c90a41cadfeeb2832e0f21e6e10d3bff8fca8bca292447b3`
passed full, Query and CLI/Text comparisons on Linux x86_64 with Rust 1.99.1:
24, 21 and 20 required stages respectively. The
[hosted Linux evidence](https://github.com/dragginzgame/ic-query/actions/runs/41843201059/job/126619913207)
binds source `126b8e044f72a714f9aace86fda287cf352479ea`; its retained summary
and all three proof/context pairs match this mapping. The same CI run passed
all three native canister lanes, including local NNS smoke and bundle admission.

The local comparison at source `075a2102e1581768d65f19e872afb1724c9d5c42`
retains its summary at
`target/release-validation/qualification.RgH8SK/qualification.json`. That source
adds documentation only and has the same mapping. Linux admission records this
observed pair. macOS 15 ARM64 also passed 24 full, 21 Query and 20 CLI/Text
stages for the same source and mapping; its
[hosted evidence](https://github.com/dragginzgame/ic-query/actions/runs/41843201059/job/126619913040)
is retained independently. Linux and macOS ARM64 are admitted. macOS Intel keeps
the full fallback until its complete comparison proof is available and admitted.
Hosted qualification remains separate from the fresh local release gate.

## Earlier mapping qualification, 2026-11-03

Actual full and selected runs passed on Linux x86_64 with Rust/Cargo 1.99.1,
using retained source capture `d164ab98f86740c75a00809e93622315905115d3`.
This is a disposable capture of the adoption batch, not a published release.
Every run used the same source, lockfile, compiler, host and configuration.

| Plan | Required stages passed | Sum of recorded stage seconds |
| --- | --- | --- |
| Full | 23 | 906 |
| Query | 20 | 385 |
| CLI/Text | 20 | 394 |

Both selected proofs omit exactly the three unchanged automation regression
suites. Required stages actually ran; stage-receipt reuse is not enabled. Compiler
artifacts were warm and other Cargo activity caused cache waits, so these totals
are observations rather than a controlled speedup benchmark.

The complete plans, contexts, checkpoints, logs, timings and proofs are retained
beneath `.git/release-state/group-qualification.wPWwrY/qualification-evidence/release-validation/`.
The historical checkout commits are preserved in the adjacent verified
`source-history.bundle`; approved retirement removed its generated source/build
copies. The comparison record is `qualification.r9VdIJ/qualification.json`. Interrupted
and failed attempts remain separate and do not grant admission.

That earlier Linux mapping was admitted. It does not qualify the later product
stage split. macOS ARM and Intel have no admitted comparison. All three hosts
retain their complete CI and canister lanes; Linux evidence never qualifies a
hosted macOS run.

The Host Tools 0.80/Platform 0.20 batch moves detail paths to the pending 0.90
notes, and [#112 — Query/CLI release plans](https://github.com/dragginzgame/ic-query/issues/112)
splits product execution into the Query/Testkit and CLI/Text stages above. These
changes invalidate the recorded mapping fingerprint. The current mapping therefore
keeps full release validation on every host until matching native comparisons
are admitted. Earlier proof remains retained; changed-mapping CI runs the new
full/selected comparison. Focused local fixtures establish caller wiring and
refusal/recovery behavior, not native admission or measured release savings.
