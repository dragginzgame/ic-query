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
| Query owner-only release | Query and private Testkit tests, CLI/Text reverse consumers, complete feature/runtime boundaries and every archive; omit only unchanged automation regressions. |
| CLI/Text owner-only release | CLI processes and Text tests against actual Query reports, public API/schema integration, complete feature/runtime boundaries and every archive; omit unchanged Query/Testkit engine tests and automation regressions. |
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
Wasm graphs, together with public report/schema consumers. Full workspace Clippy
and compilation, schema admission, dependency checks and public documentation
remain required too. Every selected plan packages all three published crates
against the real workspace versions; no package-version projection is used.
Installed Testkit/Governance and local NNS smoke checks remain in all three
native canister lanes. Source/index, publication and recovery admission remain
owned by their existing adapters.

Both groups may omit only the unchanged `ci-scripts-check`,
`publish-guards-check` and `release-guards-check` automation regressions.
Repository-wide CI retains every stage. New stages default to required; unknown
exclusion targets or groups refuse rather than silently dropping an obligation.

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
The focused UTF-8 case selects a non-ASCII omission reason through Query’s actual
selector, shared runner and retained-receipt check. Select it with
`IC_QUERY_RELEASE_VALIDATION_CASE=utf8`; the default fixture selection is unchanged.
Generic schema/path/event tests remain upstream. These fixtures establish dispatch
and custody, not product execution, native macOS support or a measured speedup.

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
beneath `.git/release-state/group-qualification.wPWwrY/source/target/release-validation/`.
The comparison record is `qualification.r9VdIJ/qualification.json`. Interrupted
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
