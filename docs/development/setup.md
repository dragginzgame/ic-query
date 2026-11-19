# Development setup

The root virtual workspace contains four crates: `ic-query`, `ic-query-topology`,
`ic-query-sns` and `ic-query-cli` (`icq`). The umbrella has empty defaults and
optional direct `topology`/`sns` re-exports. The topology library supplies bounded mainnet
[placement](../design/canister-placement.md) and
[subnet observation](../design/subnet-observation.md) reports, plus
[registered subnet inventory](../design/subnet-list.md) and
[certified resource metrics](../design/subnet-metrics.md) and
[two-sample activity](../design/subnet-activity.md). The independent
[migration library](../design/migration-evidence.md) reads ICRC balances and
selected SNS neurons.
CLI process tests exercise local help, version and rejected arguments without
network calls. Running report commands does access mainnet.

Batch `batch-U5j75Zbk` published topology 0.110.9, SNS 0.105.9, umbrella
0.104.9 and CLI 0.105.25 on 2026-11-19, ending at
`2feeba560252fcc2a956dfd46b84ebc32ea16445`. All four crates.io entries were
confirmed available and non-yanked with the Toko Tooling 0.45.16 development
pin. GitHub releases use the verified annotated tags. The Linux release gate
executed product validation once and admitted reuse for the other three groups.
Cargo verified all four archives before upload, and the built CLI reports
`icq 0.105.25`. The batch took 575.0 seconds. See the
[hosted CI run](https://github.com/dragginzgame/ic-query/actions/runs/44505975294).

Batch `batch-p5r3o_Qd` published topology 0.110.8, SNS 0.105.8, umbrella
0.104.8 and CLI 0.105.24 on 2026-11-19, ending at
`f17d0d9797c3a59e01d9ad0912fce13b0f43986d`. All four versions were confirmed
available and non-yanked on crates.io, with published GitHub releases at the
verified tags. The release selects Tooling 0.45.13. The Linux release gate passed
with one product execution and three fresh reuse admissions. Cargo verified
each archive before upload. The batch took 617.6 seconds, excluding GitHub
release page creation. See the [hosted CI run](https://github.com/dragginzgame/ic-query/actions/runs/44495490469)
for separate hosted results.

Batch `batch-SFhsiU9h` published topology 0.110.7, SNS 0.105.7, umbrella
0.104.7 and CLI 0.105.23 on 2026-11-18, ending at
`a66abb9d581f4adf18325922597889ff91b9630c`. All four registry versions were
confirmed available and non-yanked; GitHub release entries were created and the
built CLI reports `icq 0.105.23`. The release selects Tooling 0.45.11,
Platform 0.34.17, Protocol 0.3.6 and Types 0.21.3. The batch took 579.4s,
with one product execution and three fresh reuse admissions. Candidate dependency
preparation took 0.10–0.11s per group with dependencies already available, before
proof sealing. Cache warmth is otherwise unrecorded; the 3.1s difference from the
previous batch is not a controlled performance comparison.
[Hosted CI](https://github.com/dragginzgame/ic-query/actions/runs/44421298183)
was in progress at readback. Concurrent ic-host-text 0.77.3 and rand_pcg 0.10.3
lock edits are preserved on local branch `deferred/query-lock-during-4511` and
restored after reconciliation; this release retains 0.77.2 and 0.10.2 respectively.

Local adoption on 2026-11-19 selects Toko Tooling 0.45.10, Platform 0.34.17,
Protocol 0.3.6 and Types 0.21.3. The Platform family was reviewed at
[`2856344`](https://github.com/dragginzgame/ic-platform/tree/2856344360e15e0bf8f7781842dbbe6e5c7cd8ef):
the selected maintenance patches require no Query API changes; optional stable
storage capabilities remain unselected. Compatible supplier floors are retained.
The existing smallvec 1.16.6 working selection is preserved.
Rust imports, test declaration placement, function spacing and the workspace
dependency catalog follow the new baseline. `make check` passes locally on Linux
x86-64: 204 tests, two re-export doctests, the umbrella feature matrix, strict
Clippy/Rustdoc, formatting and snapshot admission. This is local qualification,
not a new publication, hosted macOS result or live-network check.
The release wiring fixture passed 32 assertions, including default, explicitly
offline and explicitly online Cargo settings. The native four-group fixture
`sequence.j3WMqY` measured 119/46/45/40s for topology/SNS/umbrella/CLI validation,
with one product execution and three fresh reuse admissions. These local timings
exclude publication and hosted CI and are not a controlled performance comparison.
The subsequent 0.45.11 adoption updates the tooling marker and snapshot. Its
optional diagnostic-helper fix is outside Query’s selected payload; the installed
policy and executable payload match 0.45.10.
The same native fixture passed isolated CLI selection, inherited Cargo policy,
prepared-tool admission, missing/tampered evidence refusal, proof-sealing recovery,
corrupted-proof re-execution, environment invalidation and failed-product recovery.

Batch `batch-5JWxwAZD` published topology 0.110.6, SNS 0.105.6, umbrella 0.104.6
and CLI 0.105.22 on 2026-11-18, ending at
`d41e3a800f753920731d038cfce33b207533bed4`. All four registry versions were
verified available and non-yanked, with GitHub release entries and built CLI
output `icq 0.105.22`. The Tooling 0.45.3 batch summary measured 582.5s, 19.4s
below the preceding 601.9s delivery. It recorded four Cargo upload packages and
three reused admissions. Reused product admission took 13.91/15.35/14.18s;
proof lookup took 0.028/0.040/0.028s. Fresh observations and custody checks remained
required. These sequential deliveries have unrecorded cache warmth and are not a
controlled benchmark. GitHub release-entry creation and documentation delivery
are outside the batch interval.
[Hosted CI](https://github.com/dragginzgame/ic-query/actions/runs/44292580476)
was still running at readback. The released dependency graph retains Platform
0.34.15 and smallvec 1.16.5; unrelated working lockfile edits remain deferred.
The release run was subsequently superseded by the documentation follow-up.
[CI run 44292861726](https://github.com/dragginzgame/ic-query/actions/runs/44292861726)
passed for `172812b345bc4ba7ea9b7ddd50459b281df646be`, containing the same delivered
product code. This closes the pending hosted result for the producer; it does not
qualify Canic's separate dependency/import cutover.

Batch `batch-1DEFBf5T` published topology 0.110.5, SNS 0.105.5, umbrella 0.104.5
and CLI 0.105.21 on 2026-11-18, ending at
`67999bc706d40557c8373c831f998bd7ad6bd01d`. It delivered shared TOML-based CI
toolchain preparation and retired the unused provenance entrypoint (Query #5).
All four registry entries were verified available and non-yanked, GitHub releases
were published, and the built CLI reports `icq 0.105.21`. The actual batch took
602s, excluding GitHub release-entry creation and documentation delivery. Local
release and archive checks passed; this is not a controlled performance comparison.
The release retained Platform 0.34.15 and smallvec 1.16.5. Unrelated working-tree
updates to those dependencies were preserved separately for restoration after
reconciliation, rather than included in this CI cleanup.
[Hosted CI](https://github.com/dragginzgame/ic-query/actions/runs/44275897846)
was still running at readback.

Batch `batch-8iQvJGge` published topology 0.110.4, SNS 0.105.4, umbrella 0.104.4
and CLI 0.105.20 on 2026-11-17, ending at
`d9421a66d2eae249606ac0f1d05c6192e88c3e3f`. All four crates.io index entries
were verified as available and non-yanked, and GitHub releases were published
for all four tags. The built CLI reports `icq 0.105.20`.
The actual batch took 604s versus the preceding 601s: this delivery does not
demonstrate an overall speedup. Validation took 273s, setup 43s, metadata
preparation 86s and publication 52s. Nested candidate projection totaled 17s
and product admission 147s; these are included in validation, not additional
intervals. Product admission includes the first product execution, not just
evidence checking. One product execution supplied the three later groups through fresh
shared proof admission. These are sequential deliveries with different inputs,
not controlled benchmarks. GitHub release-entry creation and documentation
delivery are outside the batch interval.
[Hosted CI](https://github.com/dragginzgame/ic-query/actions/runs/44184841669)
was still running at readback; the local release qualification above does not
establish a hosted result. The normal checkout was reconciled with all four tags.

Batch `batch-JQkLIRLe` published topology 0.110.3, SNS 0.105.3, umbrella 0.104.3
and CLI 0.105.19 on 2026-11-17, ending at
`2860ff74fe44d10c1f634a1cfb9db90d3e5b4880`. Its actual batch time was 601s,
versus 692s in the preceding delivery: 91s shorter (about 13%). Validation took
277s versus 340s, setup 44s versus 65s, metadata preparation 81s versus 86s,
and publication 53s versus 54s. These are observed sequential deliveries, not
a controlled benchmark; hosted CI and subsequent documentation delivery are
outside the batch interval. One product execution supplied all four groups,
with one seal per group and fresh current-version CLI/source/archive checks.
The packaged CLI reports `icq 0.105.19`. Hosted CI runs asynchronously; earlier
hosted qualification below does not establish the result for this release.

Batch `batch-xEsmsSI5` published topology 0.110.2, SNS 0.105.2, umbrella 0.104.2
and CLI 0.105.18 on 2026-11-17, ending at
`61e25445a3fec94111205157921fcee955c1e33d`. The locked Platform 0.34.14 graph
passed the full product checks once; the three subsequent groups admitted that
same execution proof. Current-source checks and CLI version tests remained fresh,
and each registry archive passed publication verification. The batch took 692s,
including tool preparation, validation, metadata preparation, Git delivery and
registry publication; this is an actual delivery measurement, not a controlled
speedup comparison. Retained phase timings remain with the batch receipts.

Initial hosted CI passed the scaffold but exposed a setup-diagnostics test that
counted every artifact upload, including the new optional timing artifact.
The correction scopes its assertions to the two setup-artifact steps; all 32
diagnostic checks pass locally. This repository-only test correction does not
change the published crate sources. [Hosted CI](https://github.com/dragginzgame/ic-query/actions/runs/44160061478)
passed scaffold and native Linux x86-64, macOS ARM64 and macOS x86-64 checks for
`6d8ad9e2d616694cfffe3d14e9956cce74ab6cfa`, including the corrected diagnostics
fixtures and the full product roster. The packaged CLI reports `icq 0.105.18`.

CLI 0.105.17 moves the migration command and presentation unit tests into sibling
`tests.rs` files under directory modules. The test bodies and production logic
were compared against their original contents; only file layout and test-module
wiring changed. All 52 CLI unit/process tests and strict CLI Clippy passed before
release. Batch `batch-ANhXjUPq` then passed the full workspace gate, including
204 tests, two re-export doctests and the umbrella feature matrix, and published
the package from commit `54f60a9b99e506b0fa1c59e9a5df988db259e88c`. This also
qualifies the retained zeroize 1.9.2 lockfile update. Registry source provenance
matches the release tag, and the installed CLI reports 0.105.17 in version and
nested help output.

The family-name cutover moves the libraries from `ic-topology` and
`ic-sns-migration` to `ic-query-topology` and `ic-query-sns`, with matching crate
directories and Rust imports. CLI commands and report contracts are unchanged.
On 2026-11-17 the renamed workspace passed `make check` on Linux: all 204 tests,
strict Clippy, Rustdoc, formatting, tools and documentation admission. Library
source and fixture bytes match the pre-rename files; package file-list inspection
retains the frozen fixtures and root keys. This is local qualification, not
publication or a new hosted macOS result. Existing published names remain
historical; consumers must change package names/imports for the new releases.
The new umbrella also passed empty-default, topology-only, SNS-only and combined
feature checks, strict all-feature Clippy/Rustdoc and two re-export identity
doctests. Its empty normal dependency graph was inspected separately; workspace
feature unification alone is not evidence of an empty-default consumer build.

The four-crate delivery published `ic-query-topology` 0.110.0, `ic-query-sns`
0.105.0 and `ic-query` 0.104.0 in reconciled batch `batch-7tJD0iOR`, followed by
`ic-query-cli` 0.105.15 in reconciled batch `batch-b8WtoUdP`. Each group ran the
complete gate, including the feature matrix, and Cargo package verification.
A separate registry-only consumer passed empty-default, topology-only, SNS-only
and combined builds, including re-export/direct-import type identity checks.
Its empty-default normal dependency graph contained only the umbrella. These
are Linux consumer compilation checks, not restored legacy API compatibility.
All four registry package source identities match their release tags. The
installed CLI reports 0.105.15 in both version and nested help output.
[Release CI run 44067644439](https://github.com/dragginzgame/ic-query/actions/runs/44067644439)
passed for `62f79584be5a0f903780fa926db9a1ee6a7c8b40` on Linux x86-64 and
macOS ARM64/x86-64: 204 existing tests, two re-export doctests, the umbrella
feature matrix, strict Clippy/Rustdoc and scaffold checks. This qualifies the
released contracts and locked suppliers, not the retired legacy surface.

The next four-crate patch delivery, batch `batch-1oQy3Qae`, published topology
0.110.1, SNS 0.105.1, umbrella 0.104.1 and CLI 0.105.16 with locked Platform
0.34.13 / Protocol 0.3.5 and reconciled to
`645ad2f0144e5450fbeb7d9b868191a007e3ad38`. Each group passed the full gate and
Cargo package verification. A registry-only consumer passed empty, topology,
SNS and combined feature builds with those exact suppliers, preserving direct
and re-exported report type identities. New supplier update/management APIs
are unused; current contracts and compatible manifest floors are unchanged.
The installed registry CLI reports 0.105.16 in version and nested help output;
all four downloaded package source identities match their release tags.
[Release CI run 44072309696](https://github.com/dragginzgame/ic-query/actions/runs/44072309696)
passed for that exact release commit on Linux x86-64 and macOS ARM64/x86-64,
including 204 tests, two re-export doctests, the feature matrix, strict
Clippy/Rustdoc and scaffold checks. No new live-network qualification is claimed.

All topology reads select published Platform 0.34.17 `native-read`, Protocol 0.3.6
for Registry/placement/cloud-engine codecs, and Types 0.21.3. Product and test
graphs no longer select legacy Platform or Agent. The topology tests replay a
[portable corpus](../../crates/ic-query-topology/src/tests/README.md), packaged with
the library; they do not need a sibling checkout or direct Candid dependency.
The CLI selects published `ic-host-text` 0.77.2 for bordered tables and terminal-safe
Unicode alignment; report libraries remain independent of presentation dependencies.

Product manifests use compatible requirements: Platform `0.34.11` permits
`>=0.34.11, <0.35.0`, Protocol `0.3.2` permits `>=0.3.2, <0.4.0`, and Types
`0.21.2` permits `>=0.21.2, <0.22.0`. Cargo.lock retains the exact versions above
for Query's locked validation and releases. Downstream consumers resolve within
these ranges and the suppliers' own constraints; the manifest does not certify
every future patch. Published Query versions cannot be edited retroactively;
the relaxed requirements are delivered in topology 0.109.2, migration 0.104.3
and CLI 0.105.14.

On 2026-11-17 a temporary downstream consumer selecting Protocol `=0.3.3`
alongside both Query libraries reproduced the old resolution failure, then passed
`cargo check --locked --offline` after relaxing the requirements. Its graph
resolved one Protocol 0.3.3 shared by both libraries and the consumer. This proves
that concrete patch composition, not arbitrary future compatibility. Query's
unchanged lockfile separately passed `make check`, including all 204 tests,
strict Clippy, Rustdoc, formatting, tools and documentation admission on Linux.
The subsequent release candidate preserves the working-tree update to Protocol
0.3.3 and qualifies it through each release group's full gate. Its capture-status
fix is unused by Query; the manifest floor remains 0.3.2.

Batch `batch-iuZZhSVo` published those releases and reconciled to commit
`29d1e41f3f8f9498232e2183c16ca534e5596a19` on 2026-11-17. Each group passed
the full local gate and Cargo package verification. Registry metadata confirms
the compatible requirements, and all three package source identities match their
release tags. The registry-installed CLI reports 0.105.14 in version and nested
help output. A registry-only consumer compiled both released libraries with
Platform 0.34.11 and Protocol 0.3.2, then 0.3.3, and finally Platform 0.34.12 /
Protocol 0.3.4, always retaining one Types 0.21.2 identity. These are Linux
resolution/compile checks, not new live-network or future-patch guarantees.
[Query #1](https://github.com/dragginzgame/ic-query/issues/1) (hook adoption) and
[Query #2](https://github.com/dragginzgame/ic-query/issues/2) (supplier constraints)
are complete. Toko-specific live checks and broader neuron export remain deferred;
new work requires a concrete migration evidence or Canic decision contract.
[Release CI run 44063613925](https://github.com/dragginzgame/ic-query/actions/runs/44063613925)
passed for commit `29d1e41f3f8f9498232e2183c16ca534e5596a19`: all 204 tests,
strict Clippy and Rustdoc on Linux x86-64 and macOS ARM64/x86-64, plus scaffold
and diagnostic-fixture checks. This hosted result uses the release lockfile;
the alternative supplier combinations above were checked on Linux separately.

The subsequent Platform 0.34.11 / Protocol 0.3.2 adoption passed `make check`
on Linux x86-64 on 2026-11-17: all 204 tests, strict Clippy, Rustdoc, formatting,
tool admission and documentation links. Only those two supplier lockfile entries
changed; Types remains 0.21.2. Supplier changes were reviewed at
[`527fe363`](https://github.com/dragginzgame/ic-platform/commit/527fe363c300ebf123d3665af813e93511a3ccd8):
module moves preserve public paths, and Query does not call the added management
APIs or select `management-snapshot-fixtures`. Each library retains its independent
feature selection. No report code, JSON schema or retained evidence changes are
required. This is local adoption qualification, not publication, new hosted macOS
qualification or a new live-network observation. The release evidence below
remains bound to the previously delivered supplier versions.

The adoption was subsequently published as topology 0.109.1, migration 0.104.2
and CLI 0.105.13 on 2026-11-17. Each release group passed the complete local gate
and Cargo package verification; registry package provenance matches its release
tag. Batch `batch-dQ3FjJC1` completed and was reconciled to CLI release commit
`6892717bb33e895478ce18636c39bf3a96edfee3`. The registry-installed CLI reports
`icq 0.105.13`, including nested help. A live Dragginz balance read returned
1000 base units, 8 decimals and DKP with all three query signatures verified;
this remains a bounded observation, not consensus-certified application state.
Hosted [release CI run 44058445311](https://github.com/dragginzgame/ic-query/actions/runs/44058445311)
passed for that exact release commit on Linux x86-64 and macOS ARM64/x86-64,
including all 204 tests, Clippy, Rustdoc, setup-diagnostics fixtures and the
scaffold checks. This qualifies the released offline contracts on those hosts;
the live CLI observation above was performed on Linux only.

The requested Platform 0.34.10 / Protocol 0.3.0 adoption passed all 204 workspace
tests, strict Clippy for each affected package, Rustdoc, formatting, local links
and tooling admission on Linux x86-64 on 2026-11-17. Each library's independent
feature graph retains one Platform/Protocol/Types cohort. Protocol's management
contract changes are unused here; public topology Protocol payload identities
nevertheless require topology 0.109.0 and caller dependency
alignment. CLI syntax and report JSON are unchanged. This adoption did not rerun
live reads or establish new macOS qualification; the earlier live evidence below
used Platform 0.34.9 and Protocol 0.2.8.

The migration implementation passed `make check` on Linux x86-64 on 2026-11-16:
204 tests (139 topology, 24 CLI unit, 28 CLI process, 13 migration), strict Clippy,
Rustdoc, formatting, local links and tooling admission. This evidence describes
the working-tree implementation based on `ca8f66c202c053d686f1cab0ec1d159b778485d7`,
not a delivered release or new hosted macOS qualification. Separate feature-graph
checks confirm topology selects no SNS/ledger capability and migration selects
no topology/Registry capability. Cargo's package file list includes the migration
root key and six frozen consumer fixtures; this is not archive build verification.

Bounded live Dragginz checks read the selected anonymous ICRC account (1000 base
units, 8 decimals, symbol DKP), one exact neuron from the returned list, 200 neurons
across two pages, a separate explicit-cursor continuation and an empty exhausted
anonymous-principal filter. Successful responses carried verified query signatures;
full page budgets returned partial status as specified. These November 16 observations
do not establish current balances, atomic snapshots or complete migration coverage.
Toko-specific live checks remain unqualified. The subsequent release qualification
below includes offline migration-contract execution on macOS.
See the [migration contract](../design/migration-evidence.md) for field omissions,
deadlines and the distinction between query authentication and application certification.

On 2026-11-17 topology 0.109.0, migration 0.104.1 and CLI 0.105.12 were published
to crates.io and GitHub with registry-only suppliers. The installed registry CLI
reported `icq 0.105.12`; live Dragginz checks repeated the balance above, read
one exact neuron and retained a 100-neuron page with the expected partial exit 1.
These are bounded mainnet observations with verified query signatures, not
consensus-certified application state or a complete migration export.

## Preparation

Use Bash, GNU Make, Git, Perl with core modules and a SHA-256 backend. Install the
exact Rust 1.100.0 toolchain with rustfmt and Clippy from
[rust-toolchain.toml](../../rust-toolchain.toml), then prepare tools and dependencies:

```bash
rustup toolchain install 1.100.0 --profile minimal --component rustfmt --component clippy
make install-tools
make fetch-dependencies
make install-hooks
```

Run `make install-hooks` once in each new clone. It sets repository-local
`core.hooksPath` to the reviewed `vendor/toko-tooling/.githooks` directory.
The pre-commit hook formats selected files and rejects partial staging; the
installer refuses to replace an existing different hook installation.

Preparation may use the network and shared user tool store. Validation is locked
and offline; it does not install tools or activate hooks. `make install-host-tools`
prepares the smaller host-tool set, including `cloc`, when only LOC reporting is
needed. Existing global `toko-cloc` launchers select this checkout's reporter;
without a launcher, use `bash vendor/toko-tooling/scripts/dev/toko-cloc -d 0`.

## Certificate-time supplier

The selected delivery is topology 0.108.0 and CLI 0.105.10 to crates.io
and GitHub. Registry inspection on 2026-11-16 confirmed non-yanked Platform
0.34.7, Protocol 0.2.6 and Types 0.21.2. That delivery selected these registry versions;
no sibling checkout is required. The unchanged SNS shell is outside this batch.

Query and the CLI share published Types 0.21.2 with Protocol and certificate
authentication. Legacy Types 0.20.4, Platform 0.33.7, Agent and their identity/class
bridges are retired. The optional canister-signature and native-signed features
are not selected. The thiserror requirement remains compatible at 2.0.18.

The current hard cut adapts the supplier's delivered patch and 313-frame corpus
at revision `53efb18ccd31f4ed7ec6f611488fe27820fbd0ef`, resolving the fixture gap
in [Platform #10](https://github.com/dragginzgame/ic-platform/issues/10).
Query owns the fixed endpoint, pinned mainnet root/subnet, 20-second whole-read
deadline, 1 MiB message limit and 300-second ingress expiry. Platform authenticates
under explicit 300-second past/60-second future windows for certificate and
delegation timestamps. Redirects, proxies and transport retries are disabled.
Progress retains its stricter 4096-node final verification budget and distinct
signing/delegation times. Other native reads allow 16384 tree nodes, depth 64,
128 KiB delegation bytes and 4096 canister ranges. Retained reports stay schema 1;
public Rust supplier/error identities require consumer migration at topology 0.108.0.

On 2026-11-16 the hard cut passed 187 topology/CLI tests (139 topology, 22 CLI
unit and 26 process), strict all-target Clippy and Rustdoc with denied warnings
on Linux x86-64. Both package archives were created with `cargo package --locked
--offline --allow-dirty --no-verify`; inspection confirmed the topology archive
includes the corpus, replay helper and pinned root. Archive creation is not
archive build verification or publication. The product graph selects exactly
one Platform/Protocol/Types cohort and no Agent; no private fixture or direct
Query Candid dependency remains. Upstream transport dependencies remain owned
by Platform. The pinned root was byte-compared with the previous transport's
mainnet trust key. A live node-signature-verified query resolved the ICP ledger to
the NNS subnet at Unix second 1794861868, without claiming result certification.
Two live root-subnet progress observations advanced by 873878127 nanoseconds;
both passed certificate-time admission. These are bounded smoke observations,
not uptime evidence. Formatting, local links and tooling admission also passed.
macOS and publication qualification remain separate.

The following validation and probe records describe earlier selections, not the
current hard-cut qualification.

The registry adoption passed the complete local Linux validation stages on
2026-11-16: strict all-target Clippy, Rustdoc, 184 tests, formatting, local
documentation links and snapshot/Cargo admission. This qualifies the registry
graph and offline report contracts; hosted macOS checks and live network reads
remain separate evidence. Registry metadata contained no external path suppliers.
That validation used Host Text 0.76.0 and Toko Tooling 0.44.5. The subsequent
requested tooling adoption selects Host Text 0.77.2 and Toko Tooling 0.44.8;
its focused CLI and tooling checks are separate from that earlier release gate.
The adoption passed all 48 CLI unit/process tests, strict all-target CLI Clippy,
formatting, documentation links and snapshot/Cargo admission on Linux x86-64.
The supported updater also admitted the selected host and Rust developer tools.

Prepared topology 0.108.0 and CLI 0.105.10 archives passed a combined
`cargo publish --dry-run --locked --allow-dirty -p ic-topology -p ic-query-cli`
on 2026-11-16, including verification of the CLI against the packaged topology
crate. Final-version CLI process tests and documentation admission also passed.
No upload or Git delivery occurred in that earlier preparation. The current
[release adapter](releases.md) now exposes the required shared release and
publication entrypoints; final source and archive admission occur during delivery.

The subsequent registry update to Platform 0.34.6, Protocol 0.2.5 and Types
0.21.2 passed all 184 topology/CLI tests, strict all-target Clippy, formatting,
documentation links and snapshot/Cargo admission on Linux x86-64 on 2026-11-16.
Only these three supplier packages changed in the lockfile. Adoption remains
partial: legacy Platform 0.33.7 still supplies transport, remaining readers and
fixture encoding, and the portable-fixture issue below remains open.

The earlier Platform 0.34.5 / Protocol 0.2.4 hard-cut probe compiled topology
and the CLI against registry packages on Linux x86-64, with one Types 0.21.1 and
no Agent or legacy Platform edge. It adapted the supplier's
[consumer patch and qualification](https://github.com/dragginzgame/ic-platform/blob/1e92703bc803b81dbe1e3d815fd1077d440d62b4/docs/qualification/query-hard-cut.md)
in disposable validation storage only; the working implementation is not migrated.
The supplier patch requires its unpublished `engine-view-consumer` fixture crate
for availability, prices, engine and placement tests. Removing that sibling
dependency makes test compilation fail with unresolved fixture imports. Completing
the cut requires portable fixture inputs or a published supplier fixture API,
tracked in [Platform #10 — Provide portable fixtures for Query's registry-only Platform hard cut](https://github.com/dragginzgame/ic-platform/issues/10),
then qualification here; the supplier's passing local-path run does not establish
registry-only consumer qualification. Do not retain the legacy dev dependency,
import a direct Candid dependency or remove regression coverage to bypass this gap.

The earlier local identity migration passed 184 workspace tests and strict all-target Clippy
on Linux x86-64. The supplier's subsequent 0.34.1 version alignment is requalified
through focused progress tests and Rustdoc. This does not qualify the moving
sibling checkout on macOS or for delivery. Existing report schemas and raw
principal bytes are preserved; Rust identity consumers must align Types and rebuild.
Live read-only checks with local Platform 0.34.1 resolved the ICP ledger to the
NNS subnet and authenticated two advancing NNS state times on 2026-11-16. These
check the legacy Registry/new certificate boundaries, not all live report sources.

## Daily commands

```bash
make help
make icq
make deps-list
make deps-update
toko-cloc -d 0
make qualify-cli
make qualify-umbrella
make test PACKAGE=ic-query-topology
```

`make icq` builds and installs this checkout's CLI in Cargo's installation bin
directory (normally `~/.cargo/bin`), replacing an existing Cargo-installed `icq`.
It uses the development profile and prepared, locked dependencies, then checks
that `icq` is available on PATH. Cargo installation-root overrides are respected;
keep the selected bin directory on PATH. Run `icq --help` or pass report arguments
directly. Cargo may fetch missing locked dependencies; report commands access mainnet.
For an explicitly offline development run, use `CARGO_NET_OFFLINE=true make check`
after preparing dependencies. Ordinary commands preserve caller Cargo settings.

`deps-list` lists local workspace package versions, not transitive dependencies
or registry publication status. The group column reads the ownership declared in
[`ci/cargo-release-groups.json`](../../ci/cargo-release-groups.json). Release
delivery uses the shared runner and the local release adapter. `deps-update`
runs the shared `cargo update -v`
workflow; it intentionally changes lockfiles and may access the registry. To select
one dependency, use `make deps-update DEPENDENCY_PACKAGE=clap`. Review direct
supplier freshness separately: a lockfile update does not widen manifest ranges.

The tooling source is pinned exactly in the root catalog and inherited as a
dev-dependency by every member, with the prescribed machete exemption. Use the
[tooling selection rule](https://github.com/dragginzgame/toko-tooling/blob/a716b501274678a8ebf163ef576bbb549e974acd/rules/cargo-dependencies.md#toko-tooling-during-dependency-updates)
if a supported check reports the unused marker. To adopt a new tooling release,
update the pin and lockfile, run `make toko-tooling-preview` and review the
selection before running `make toko-tooling-update`.

## Focused validation and qualification

```bash
make qualify-cli
make clippy PACKAGE=ic-query-topology
make rustdoc PACKAGE=ic-query-topology
make test PACKAGE=ic-query-cli
make fmt-check
make doc-links-check
make toko-tooling-check
```

Use the affected package or CLI test target during ordinary development.
`make check` / `make ci` are aggregate gates: snapshot/Cargo admission, local links,
formatting, Clippy, Rustdoc and tests. `PACKAGE=name` narrows Cargo compilation,
documentation and test selection; formatting and documentation-link checks remain
repository-wide. All four packages are covered by the default aggregate, including
the umbrella's empty, topology-only, SNS-only and combined feature configurations.
With `PACKAGE=name`, only that package's Rust gates run; use `qualify-umbrella`
explicitly when changing its feature combinations.
Warnings are errors in the Rust gates. `make fmt` applies the shared formatter.

### Release validation selection

[Query #3 — Select affected Query packages and consumers](https://github.com/dragginzgame/ic-query/issues/3)
is addressed by the local selector over Tooling's shared baseline, plan and
evidence helpers. The [stage roster](../../ci/release-validation.json) is used
once by `make check`, by native CI (without the separate scaffold checks), and
by release selection. Group membership remains in the Cargo release policy.

| Isolated implementation change / selected group | Required Rust checks |
| --- | --- |
| CLI | CLI Clippy, Rustdoc, unit and process tests |
| Umbrella | Empty, topology, SNS and combined feature checks; combined lint, docs and re-export doctests |
| Topology | Topology, CLI and umbrella checks |
| SNS migration | SNS, CLI and umbrella checks |
| Shared, unclassified or unavailable baseline | Complete roster |

Selection includes **all** changed paths since the selected group's previous
annotated ancestor tag, plus the group's own checks. A specialist change broadens
a CLI release even when it has not been released separately. Manifests, locks,
tooling, workflow/recipe changes, design/setup documentation and unknown paths
select the full roster. Release notes and the root README do not broaden Rust
selection. The selector refuses an unmapped stage or group; new obligations must
update the mapping and its fixtures. Baseline availability alone grants no reuse.

| Flow | Owner, guarantee and trigger |
| --- | --- |
| Development | `make clippy/rustdoc/test PACKAGE=name` selects the edited package; `make check` checks the complete roster |
| Changed release wiring | `make qualify-release-validation` exercises Query mappings and actual Make/Cargo argv with simulated Cargo effects; also selected in scaffold CI |
| Repository CI | Scaffold owns formatting, docs, secret scanning and wiring fixtures; `make validate-native` runs every product stage on all three supported hosts |
| Release/batch | `scripts/release.sh verify` selects from the previous group tag, admits Linux candidate product proofs where eligible, and writes a source/group-bound receipt |
| Publication | Existing group/tag/source admission and selected-package archive verification, upload and registry provenance remain fresh |
| Recovery | Completed same-source validation is re-admitted from its original runner records; missing proof output can be reconstructed without executing Cargo again |

Release evidence lives under `target/validation/release.*`, referenced by protected
`.git/release-state` receipts. Keep these records with the release attempt.
The shared verifier rejects missing, changed, failed or interrupted group evidence.
The receipt binds the complete proof and logs across the separately admitted
version-preparation transaction. Original full-gate receipts retain their old
meaning for pre-selection source commits. A failed/incomplete gate starts a new
retained attempt; an already executed product proof may satisfy the new attempt
only after shared exact-input admission. New source selects a fresh plan and
preserves the previous attempt.
A completed gate whose evidence is damaged refuses rather than silently
replacing its proof or rerunning expensive work.

On Linux with the admitted system GCC tool closure, Tooling's canonical editor
prepares a disposable source containing the
intended final versions from the numbered group drafts. Query runs the selected
Rust checks as one product set in that source. A later group can reach the exact
same candidate bytes after earlier groups have shipped, and reuse the original
execution proof. Source, selected targets, dependencies, compiler/native inputs,
executable bytes, bounded environment or host changes invalidate the match.
Different selections execute separately; a full-set proof is not relabelled as
an independently executed subset. This avoids repeated native-input observation
for each small package stage without adding a local proof cache or version editor.

The candidate release plan now retains that selection in `obligations.json` and
projects it into `validate-common`, one `validate-products` admission and, when
selected, `validate-current-cli`. Tooling checkpoints both the obligation map and
the projected execution policy. The combined product proof is sealed once per
group; it is not resealed for each specialist/umbrella label. Full development,
native CI and non-candidate fallback still use the original complete stage roster.
Existing completed release evidence retains its original meaning.

Setup fetches missing locked dependencies and checks the selected tool payloads.
Candidate dependencies are prepared before reusable evidence is sealed. Explicit
`CARGO_NET_OFFLINE` settings are preserved during setup and product execution.
Fresh tool preparation installs only after a failed availability check; recovery
keeps tool admission check-only. Group boundaries still recheck tools, source
and metadata: no batch marker or cached success bypasses those admissions.
The shared timing journal now separates candidate/editor preparation, native tool
inventory, product admission, seal write/check, current-version CLI tests, plan
finalization and receipt finalization. Parent spans include their children and
must not be added to them. This addresses the repeated work tracked in
[Query #4](https://github.com/dragginzgame/ic-query/issues/4).

The local Linux four-group comparison on 2026-11-17 used baseline `8f62960`
and the pending Query #4 implementation, with cached dependencies and fresh
candidate build directories in each fixture. Both ran the same product versions
and locked Platform 0.34.14 graph; metadata delivery was simulated. Validation
times were 145/72/70/66s before and 126/53/50/46s after (topology/SNS/umbrella/CLI):
353s to 275s across the sequence, a 78s reduction in this single local comparison.
This excludes metadata preparation, Git delivery, publication and hosted CI;
it is not a measured reduction of the earlier 692s real deployment.
Each new group wrote one seal, with no repeated per-product seal checks.

On the reused SNS group, candidate preparation took approximately 11s, combined
product admission 20s (including the shared runner's 8s interval), sealing 3s,
current-source CLI tests 4s, and plan finalization 1s. Common checks and dispatch
account for the remaining interval. Native tool inventory and receipt finalization
were each below one second. Canonical source projection and shared proof admission
remain significant; these observations do not classify their work as removable.
The 31 focused wiring assertions also pass, including preservation of all four
group mappings under the combined plan. Raw fixtures, phase journals and logs stay
under `target/reuse-qualification/` and `target/validation-fixtures/`.
The native qualifier passed missing/tampered proof, log and admission refusal,
changed obligation/input refusal, failed seal-output recovery, corrupted-proof
re-execution, environment invalidation and failed-product recovery. Its final
full-selection admission reused the original proof in 45s. Prepared-tool checks,
missing-tool preparation and check-only recovery passed with installation effects
simulated. Workflow lint and all 32 setup-diagnostic assertions passed as well.
Optional hosted qualification retains compact `phases.tsv` fields alongside
group timings; it does not upload tool inventories or environment values.
These are local qualification results; the subsequent actual delivery timings
are recorded at the beginning of this guide.

Receipts retain `executed` or `reused`, original execution source/group, logs and
shared custody bindings. Missing/stale product proof takes ordinary execution;
corruption keeps its diagnostic and damaged proof while the shared runner seeks
a valid match or executes anew. A failed command produces no successful proof.
The source renderer and admission helper never publish from the candidate.
Actual group metadata, selected package archives, registry availability and
upload/readback recovery retain their existing fresh checks.

Focused shared-operation profiling on 2026-11-17 used installed Tooling 0.44.11
and an isolated archive of `faf907b2c73aa1e007ddc1224a5dd202a2c65896`, retaining
the delivered Platform 0.34.14 lockfile. The canonical renderer produced candidate
`7b40a0d9812caa0668925884ca3ef31682d98422ba59bfa32798416a037e3b8f`.
Transparent child-command timers and temporary Perl function wrappers measured
the unchanged shared helpers; controls omitted both forms of instrumentation.

| Operation | Instrumented wall time | Uninstrumented control |
| --- | ---: | ---: |
| Four-group candidate preparation | 10.02s | 9.53s |
| Initial synthetic native proof | 25.38s | 25.64s |
| Reused synthetic native proof admission | 20.21s | 20.28s |

Candidate preparation made 12 full graph admissions: four group plans and
before/after admission for four prepares. Twelve `yq` lockfile decodes consumed
5.33s; twelve Cargo metadata calls consumed 1.46s; 140 `jq` calls consumed 0.87s.
Sixty manifest decodes were individually below the timer's 0.01s resolution.
The control produced byte-identical projection output. Shared batch admission
and parsing improvements are tracked in
[Tooling #390](https://github.com/dragginzgame/toko-tooling/issues/390).
The actual delivery's 81s aggregate `release-prepare` interval also includes
preflight, version reads, receipt checks and prepared checks; it is not a measure
of metadata editing alone.

The native probe used `/usr/bin/true`, contract `profile-only/1`, separate new
proof namespaces and the real 26,359-path native input observation. It did not
run or qualify product tests. Instrumentation changes verifier runtime identity,
so profiled and control runs each created their own original proof, then reused
that proof. Delivered evidence remained untouched. In the instrumented reused
run, initial and final observations took 8.07s and 8.05s. Across the parent
process, SHA-256 calls consumed 8.64s, byte reads 2.77s, JSON decoding 4.36s and
encoding 1.36s; log checks took less than 1ms. These inclusive function times
overlap the observation intervals and must not be added to them. Observer child
time is included in observation totals, without separate attribution.
The shared runner starts its existing stage interval after initial observation
and proof lookup, explaining much of the outer/inner timing difference.
[Tooling #391](https://github.com/dragginzgame/toko-tooling/issues/391) tracks
finer shared timing and qualified serialization/hashing improvements.

These are single-host Linux samples with warm dependencies, not measured future
deployment savings. Hashing cost does not establish that integrity checks can
be skipped. No local cache or supplier implementation change was introduced.
Raw profiles and disposable fixtures remain under
`target/evidence/query-admission-profile/run.brYM8z/`; the table above retains
the compact results without tool inventories or environment values.

The 2026-11-18 read-only investigation of the 0.45.0 delivery used the existing
`batch-8iQvJGge` timing journal and retained stage records. Its 147.45s product
admission aggregate breaks down as follows (seconds; observations and lookup are
children of the outer interval, not additional elapsed time):

| Group / mode | Outer admission | Initial observation | Post-execution observation | Proof lookup | Final observation |
| --- | ---: | ---: | ---: | ---: | ---: |
| Topology / executed | 80.96 | 7.32 | 7.34 | 0.00 | 7.45 |
| SNS / reused | 18.22 | 7.35 | — | 3.11 | 7.26 |
| Umbrella / reused | 30.11 | 19.09 | — | 3.16 | 7.36 |
| CLI / reused | 18.15 | 7.26 | — | 3.09 | 7.30 |

Nine observations total 77.73s. Original execution evidence records 65.49s,
including its 7.34s post-execution observation: roughly 58.15s represents product
execution and associated bookkeeping. Reused admissions total 66.49s. Proof
lookup totals 9.36s, context serialization 0.75s and final custody checks 0.79s.
The umbrella initial-observation stderr records Cargo package-cache lock waits;
the journal does not isolate their duration. In the latest `batch-1DEFBf5T`, the
three reused admissions were 18.00/18.08/17.48s, so 30.11s is not a stable cost.

All nine observer documents describe the same 26,360 paths; all four retained
initial contexts match. The roster comprises 235 candidate paths, 13,235 registry
paths, 4,074 Rust toolchain paths, 4,905 system headers and 3,911 other native/tool
paths. The runner hashes those paths at each boundary: 237,240 listed-path digest
reads, plus runtime and proof/log reads. The observer independently hashes the
235 candidate source files on each invocation. There are 26,331 distinct resolved
paths, with 29 additional alias entries. Each reused group also parses the same
approximately 7.6MB canonical proof before the final custody recheck.

These counts identify repeated work, not permission to skip fresh admission.
The journal cannot attribute time or bytes to individual file reads. Query calls
the shared runner once per group and has no redundant local dispatch to remove.
[Tooling #394](https://github.com/dragginzgame/toko-tooling/issues/394) owns further
profiling and qualified reductions within shared observation/parsing operations,
preserving logical paths, mutation checks, before/final observation and original
proof custody. No local cache, runtime change or new speedup is claimed here;
historical proofs and the unrelated working lockfile remain untouched. Hosted CI
completion is independent of these local admission costs and is not a reason to
remove a required release check or add a hosted wait.

Common source/tool/format/document admission still runs on every actual group.
When CLI checks are selected, its 28 process tests also run against the actual
intermediate versions. Candidate tests alone cannot establish that installed
version/help contract. Cargo configuration/build overrides, new library package
metadata readers, build scripts or unqualified native tool layouts use ordinary current-source validation until
their effects are qualified. The current reuse contract also excludes product
reads of release ledgers; none exist. macOS retains ordinary execution because
the shared native-input observer currently qualifies Linux only. CI still tests
all supported hosts, independently of local publication.

```bash
make qualify-release-validation
make check
# Explicit, slower native sequence and adversarial recovery qualification:
make qualify-release-reuse
# Four-group baseline with fresh candidate artifacts and cached dependencies:
bash scripts/ci/test-release-reuse.sh --baseline \
  8f62960
```

The rewritten full gate passed locally on Linux on 2026-11-17: 204 tests, two
re-export doctests, all four feature configurations, strict Clippy/Rustdoc and
common admission. Its warm local stage timings were 6s common, 15s topology,
5s SNS, 7s CLI and 5s umbrella; some Cargo artifacts were rebuilt. These are
stage timings, not a cold-build benchmark or a measured deployment speedup.
All 26 wiring assertions passed locally on Linux. They cover the four mappings, actual leaf arguments, conservative
fallback, source drift, required failures, metadata-era custody and recovery.
They simulate Cargo and do not publish, exercise registry upload,
or establish hosted macOS acceptance for this pending patch. The separate native
reuse qualifier runs real Cargo in a retained fixture, prepares all four groups
through the canonical metadata transaction, and checks cross-group proof origin,
isolated CLI dispatch, damaged evidence, bounded-environment drift, required
failure and recovery. Git commits in that fixture simulate metadata delivery;
no tag push, upload or registry effect is performed. Its logs and phase timings
remain under `target/reuse-qualification/`. Run it when the mapping, source
projection, native input contract or relevant release wiring changes, or select
`qualify_release_reuse` in the CI manual-dispatch inputs. It is not an extra gate
on every release or ordinary CI push.

Local native qualification on 2026-11-17 passed the four-group sequence:
topology executed the selected product checks (148s total admission), then SNS,
umbrella and CLI reused the same original execution proof (70s, 67s and 64s).
The subsequent conservative expansion of the native tool inventory was qualified
with isolated CLI execution, seal-output recovery, corrupted-log rejection,
environment invalidation and failed-build refusal/recovery. A full admission
with that final inventory executed fresh product checks in 109s; the ordinary
full gate with warm dependencies took 54s. A subsequent umbrella-group admission
reused that final full-product execution proof and passed in 68s, including fresh
common checks and actual-version CLI tests. These paths provide different evidence
and do not demonstrate a wall-clock speedup: input observation, proof custody
and fresh current-source checks have measurable overhead. All observations were
local Linux fixture runs; hosted acceptance and actual publication remain pending.

CLI integration tests belong in `crates/ic-query-cli/tests/`. Add source fixtures
beside the owning library as reports are implemented. No separate qualification
package is justified yet: when a real external consumer, simulator, benchmark or
other test-only package is needed, put it under `qualification/`, create its own
workspace and lockfile, and exclude it from the product workspace. The shared
`deps-list`, `deps-update` and LOC reporter already discover that standard location.
At that point extend fetch, formatting and validation to the second workspace.

CI checks the implemented packages on Linux x86-64 and macOS ARM64/x86-64. Formatting and a
single pinned Gitleaks history scan run separately from the host matrix. Every job
admits snapshot bytes before preparing host tools and Rust, then admits the Cargo
pin, members and lockfile before validation.

[Release CI run 44053372936](https://github.com/dragginzgame/ic-query/actions/runs/44053372936)
passed for commit `76c94fba5776b95d7be88602f4dba721c47f9763` on 2026-11-17:
Clippy, Rustdoc and all 204 tests on Linux x86-64 and macOS ARM64/x86-64,
plus scaffold formatting, documentation/tooling admission and secret scanning.
This qualifies the delivered offline topology and migration contracts with
Platform 0.34.10, Protocol 0.3.0 and Types 0.21.2; it does not qualify live reads
on macOS or any later changes.

### Setup failure evidence

CI wraps setup commands with [the local collector](../../scripts/ci/setup-diagnostics.pl).
Both scaffold and native jobs prepare host tools first, then expose
`.tools/host/bin` on the invoking step's `PATH` for `yq` and `jq`. They call the
snapshot-owned `prepare-rust-toolchain.sh --toolchain-file rust-toolchain.toml`
through that collector. Channel, profile, components and optional targets come
from the TOML file, without a second workflow selection. Local workflow lint,
all 32 diagnostic assertions and simulated rustup calls from both actual step
bodies passed; a simulated status 29 retained diagnostics and remained failed.
This does not establish a new hosted setup result.

It preserves the original failing status and prints sanitized excerpts. Failure
artifacts contain at most 64 KiB of command output and eight selected attempt logs
from the runner's dedicated Query tool/security directories, including hidden
attempts. Each log reads at most 16 KiB and retains at most 64 KiB after sanitization;
traversal stops at 1000 entries per root. Oversized lines, known credentials,
sensitive fields, URL credentials/query strings and private-key blocks are omitted.
Symlinks, caches, tool payloads and environment dumps are excluded.

Each job uploads only the sanitized evidence directory on setup failure, retains
it for seven days and links the artifact in its summary. Missing evidence and
collection/upload failures are reported separately from the original setup error.
The focused offline fixtures exercise hidden logs, sanitization, bounded output,
missing logs, collector failure and original exit status:

```bash
perl scripts/ci/test-setup-diagnostics.pl
actionlint .github/workflows/ci.yml
```

The 32 fixture assertions and workflow lint passed locally on Linux. They qualify
local capture and wiring; hosted failure-artifact upload/retrieval has not been
observed. A successful hosted setup run does not exercise that failure path.

[CI run 43704031732](https://github.com/dragginzgame/ic-query/actions/runs/43704031732)
passed on 2026-11-15 for commit `263a8e92da873543825c25e7dd6530c4db5ab673`:
Clippy, Rustdoc and tests succeeded on all three hosts; formatting, snapshot and
documentation admission, and the secret scan also passed. This qualifies the
implemented offline contracts at that commit, including distribution, cloud-engine
reports, maintenance and two-subnet comparison, with Toko Tooling 0.44.3.
It does not qualify the subsequently implemented subnet-settings report or later changes.
The local Linux working tree subsequently passed the full `make check` gate on
2026-11-15 for registered settings and optional comparison settings: 175 tests,
Clippy, Rustdoc, formatting, snapshot admission and local links. This used the
Protocol pin recorded above and existing package versions, with pending topology
0.107.0 / CLI 0.105.9 notes. It is neither hosted macOS qualification nor GitHub
delivery of those candidate versions.
For the 2026-11-16 delivery, the full local Linux gate passed with the immutable
Platform selections: 181 tests, Clippy, Rustdoc, formatting, snapshot admission
and documentation links. Final topology 0.107.0 / CLI 0.105.9 metadata then passed
CLI process qualification and dependency/format/link admission. Cargo metadata
contains no external path dependency. Native macOS CI remains a separate result.
Broader protocol, retained-data
migration and publication qualification remain future work tied to implemented contracts.
Placement fixtures qualify decoding, projection, bounds and failures; they do not
prove cryptographic verification or live network availability. Run any live smoke
check explicitly and record its endpoint and result separately from offline tests.

## Tooling provenance

Independent groups own their release notes in `releases/<group>/CHANGELOG.md`;
root `CHANGELOG.md` is the navigation index and includes a short `▶️ Try it`
section identifying the affected CLI group/version. Keep setup, runnable commands
and expected results directly in that root quick start, with detailed examples
in the [CLI ledger](../../releases/cli/CHANGELOG.md), using actual public IDs.
This follows the installed baseline and its pinned
[group changelog policy](https://github.com/dragginzgame/toko-tooling/blob/a716b501274678a8ebf163ef576bbb549e974acd/rules/changelogs.md#release-group-changelogs-and-shared-history).

The current checkout selects Toko Tooling **0.45.16**. The exporter verified
the crates.io archive against the lockfile and used the 0.45.13 archive to
verify the previous snapshot. The refresh updated the installed baseline and
receipt. Query's Rust source uses `cfg(test)` for unit-test modules and feature
gates for the umbrella's optional exports. The source and manifests have no
host or Wasm target guards. The current archive SHA-256 is:

```text
a2095850545e0fd3c5580fab610cab78561242fc09c74221e27f3e7f654d5776
```

The [receipt](../../.toko-tooling.snapshot) records the selected roots and
all required executable companions: baseline, canonical root files, CI tool
installer, documentation link checker, download-cache action, common Make
interface and Cargo dependency Make interface. The common interface brings the
LOC reporter. Reference documents and producer regression suites stay upstream.
The current selection includes the pre-commit hook and 84 installed payload files,
including the explicitly selected Rust toolchain preparation helper and its
workspace-version reader companion.
Release/publication runners and source/receipt admission remain selected.
The unused standalone package-provenance entrypoint was retired through the
authenticated exporter after tracing Make, CI, hooks, scripts, documentation and
shared companion declarations. Publication still uses `scripts/release.sh
publication` for source/tag admission, then the shared Cargo publication adapter
and runner for locked package verification and upload. No new duplicate archive
check was added. This resolves
[Query #5](https://github.com/dragginzgame/ic-query/issues/5).
The earlier 0.45.0 exporter updated grouped candidate preparation, Cargo metadata
admission and the shared proof runner. Invocation-local TOML reuse compares full
input bytes; fresh graph checks remain around mutations. Streaming input hashes
and reduced canonical serialization preserve proof custody. Detailed runner
spans now separate observation, lookup and custody work. These are the supplier
changes requested in Tooling #390 and Tooling #391 above; Query adds no local
cache or alternate metadata/proof implementation.
Local adoption checks passed snapshot/member/lock admission, installed tool
checks, documentation links and all 31 release-wiring assertions. The native
four-group fixture at `target/reuse-qualification/sequence.6Mr23i/` passed with
126/55/52/50s validation intervals (topology/SNS/umbrella/CLI): topology executed
the product checks and the other groups reused its proof with unchanged candidate
identity. This run used the working lockfile's Platform 0.34.15 and bytes 1.12.2;
it is not a controlled comparison with the earlier Platform 0.34.14 profile or
an actual deployment measurement. Metadata delivery was simulated locally.
The setup/recovery portion resumed in the same fixture after the sandbox refused
the shared tool-store installation lock; broader filesystem access admitted that
setup without repeating the completed group sequence. It passed prepared-tool
and check-only recovery, isolated CLI selection, missing/tampered evidence
refusal, seal-output recovery, corrupted-proof re-execution, environment
invalidation and failed-product refusal/recovery. Final complete admission reused
the original product proof in 46s. No tag, push or registry upload ran.
The new flow guidance matches the development, CI, release, publication and
recovery ownership table above. Versions 0.44.10–0.44.11 introduced setup-failure
diagnostics and repository-qualified issue references; Query's implemented
diagnostic coverage and its limits are recorded in the setup failure section.
Version 0.45.3 reduces repeated canonical JSON processing in the shared runner:
unchanged observer output can reuse its parsed inventory within an invocation,
while every observation still resolves and hashes the listed files. Proof lookup
matches the freshly computed canonical context bytes and validates the remaining
envelope, avoiding a second decode of that context. New spans separate observer
commands, inventory parsing and listed-file hashing. The existing Cargo sealer
also receives the supplier's stricter record admission; Query does not select
the new generic non-Cargo mode or add a local proof cache.

The shared batch summary reads retained journals without replaying a release:

```bash
make release-batch-summary BATCH=batch-1DEFBf5T
```

This requires that named batch's local retained evidence. The adoption check
reproduced 601.9s versus 603.7s for its preceding matching batch and verified the
intent and timing journal hashes were unchanged. Older journals lack the new
Cargo package-count observations, so the summary correctly reports those counts
as unrecorded. This is historical readback, not a 0.45.3 deployment benchmark.

Local 0.45.3 adoption qualification on 2026-11-18 passed snapshot/tool admission,
documentation links, workflow lint, all 32 diagnostic assertions and all 31
release-wiring assertions. Native fixture `sequence.US8m4y` passed the four groups
in 126/43/41/40s with one execution and three reuses of the same candidate proof.
It also passed isolated CLI dispatch, missing/tampered evidence refusal,
seal-output recovery, corrupted-proof re-execution, environment invalidation,
failed-product recovery and setup/check-only recovery. Final complete admission
reused the proof in 38s. Metadata delivery and installation effects were simulated;
no publication ran. The fixture used the existing working selections of Platform
0.34.16 and smallvec 1.16.6, with package-cache waits observed, so these are not
controlled comparisons with the published 0.45.0 batches.

The SNS reuse sample took 13.96s for product admission. Its initial/final observer
commands took 0.95/0.95s, listed-file hashing 5.32/5.27s, inventory JSON processing
0.91/0.003s and proof lookup 0.028s. These nested intervals demonstrate the new
timing breakdown and retained hashing; they do not establish a deployment speedup.
The supplier's exact-byte JSON reuse operates inside the shared runner, with no
Query-owned cache or weakened admission boundary.
The baseline also clarifies asynchronous hosted
CI for delivery, one-tier qualification package layout and independent compatible
thiserror selection. Query has no separate qualification packages, and its
thiserror requirement is compatible rather than an exact fleet-wide hold.
Version 0.44.9 makes named-supplier adoption cover product, dev/build and
qualification callers by default. Query's maintained manifests and lockfile
were inventoried; no independent qualification workspace exists. Platform
adoption was partial at that revision; the current hard cut above adopts the
fixture delivery from Platform #10.
The working-tree update to Platform 0.34.5, Protocol 0.2.4 and Toko Tooling
0.44.9 passed 184 topology/CLI tests, strict all-target topology Clippy,
formatting, documentation links and snapshot/Cargo admission on Linux x86-64.
It still selects Platform 0.33.7 for transport, remaining readers and fixture
encoding; the earlier registry-only hard-cut probe is separate evidence.
The LOC reporter now defaults to repository totals: `toko-cloc` and `toko-cloc -d 0`
produce the same report; use `toko-cloc -d 1` to expand crates and `-d 2` for
top-level modules. Named crate/module selections expand immediate children at `-d 1`.
Local verification compared both default-depth outputs byte-for-byte successfully.
This checkout has no imports or
snapshot selection for the retired Python evidence API, so no caller migration
is needed. The exporter refreshed the receipt and admitted installed host/Rust
tools. The Cargo pin, lockfile and receipt identify the authenticated release.
Version 0.44.2 adds the baseline's focused post-push review before the next
increment. Use the delivered commits and directly affected callers, retain
existing validation evidence and record actionable findings in existing owning
records. It adds no separate audit framework or automatic follow-up push.
Version 0.44.3 changes only the selected LOC reporter and classifier; the baseline
is unchanged from 0.44.2. Its depth correction leaves JSON counts unchanged.
Version 0.44.4 adds package-directory/name alignment and copyable long-command
continuations to the baseline; executable payload files are unchanged. All three
maintained packages already match their directories, the root is virtual, and
the CLI examples follow the continuation convention. No package moves or product
API changes were required. The supported updater refreshed the authenticated
snapshot and admitted the installed host and Rust tools.

See the revision-pinned
[consumer adoption guide](https://github.com/dragginzgame/toko-tooling/blob/a716b501274678a8ebf163ef576bbb549e974acd/docs/adoption/consuming-snapshots.md)
and [Make interface](https://github.com/dragginzgame/toko-tooling/blob/a716b501274678a8ebf163ef576bbb549e974acd/docs/adoption/make-interface.md).
Refresh through the authenticated exporter with the complete retained entrypoint
set and review its preview. Never edit managed files or the receipt by hand.
