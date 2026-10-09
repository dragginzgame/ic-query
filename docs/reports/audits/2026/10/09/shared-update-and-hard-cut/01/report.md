# Shared update and hard-cut audit — 2026-10-09, run 01

## Scope and identities

The maintainer requested the new Shared Tooling review and an audit considering
a hard cut. Primary method: `audits/complexity-and-technical-debt.md`, with
`audits/flow-convergence-and-duplication.md` tracing and the `audits/README.md`
evidence contract, selected from snapshot
`9af82393c620e486578febed74a648523725c234`. The engineering baseline remains
`5a65686d1da9e039b22b3e17694ea083d82cfa9e`.

Query HEAD is `edd68020d9ef8e76a241e89daf77e51155a92f57`, package 0.50.3.
The existing partly staged 0.50.4 snapshot/notes batch and unstaged host-matrix
update were preserved. Current workspace membership is two crates, `ic-query`
and `ic-query-cli`; three published Host 0.8.4 packages are selected, with process
support development-only. The four Host packages are external owners rather
than four Query workspace crates. Host source remains
`97187b2a46d6f8a6964224a36a133d858ef0d223`.

Mode: source inspection and read-only GitHub observations, with this report and
its artifacts as the only new repository files. No dependency/snapshot update,
source fix, build, test, network probe, installation or release was executed.
See [initial state](artifacts/worktree-before.txt),
[pending batch](artifacts/pending-batch.txt) and
[source identities](artifacts/source-sha256.txt).

Scope includes development smoke execution, cache-refresh announcement
ownership, Candid reply admission, workspace feature boundaries and the newly
committed Shared changes. Generated protocol definitions, cryptographic
algorithms, individual report semantics and performance are outside this
structural conclusion. Native coverage for a proposed replacement is a required
gap, not an excluded obligation. This is a provisional local audit scope grounded
in AGENTS.md and cache/smoke design contracts, not adoption of a new audit overlay.
The older 2026-10-08 audit used a different source/scope; aggregate comparisons
are N/A. Its leaf-request and typed-attempt findings have current concrete owners.

## New Shared source

Remote main and the clean sibling checkout identify
`635a39a9dd5f8d021fa9c9196b591e00521a7e02`, committed as “0.1.32”. Its
`VERSION` remains 0.1.31, the 0.1.32 notes are undated, and no matching v0.1.32
tag was observed: this is committed preparation, not a verified finalized release.
The [source diff](artifacts/shared-031-to-032-source.diff) is retained.

Within Query's 74-file selection, only IC-tool, local-setup and verification
guidance changes. Selected runtime helper bytes and pin records are unchanged
from the prepared 0.1.31 adoption. Owner changes preserve each pushed source's
CI, fix Cargo qualification path/profile admission, add its fixture and adjust
owner regression limits. Query already has source-specific push concurrency;
owner assessment and fleet fixtures are outside its selected snapshot.
The exact [CI run](https://github.com/dragginzgame/shared-tooling/actions/runs/37893234402)
was in progress when inspected. No new consumer-native result is inferred.

PocketIC selection/provisioning moves toward Testkit under
[Shared #76](https://github.com/dragginzgame/shared-tooling/issues/76) and
[Testkit #38](https://github.com/dragginzgame/ic-testkit/issues/38). Setup/check
is reported implemented locally, but publication and both native macOS hosts
remain outstanding. Shared still requires six tools per host. Query dispatches
ICP at project/build/deployment/managed-runtime boundaries; no standalone
PocketIC execution was found. Do not remove shared rows, weaken checking, add a
consumer downloader or acquire a Testkit dependency solely to replace an unused
standalone binary. Trace the eventual shared setup change separately from ICP's
managed-network project configuration and preserve existing bundles/receipts.

## State and ownership trace

| Behavior | Owner → carried contract → projection | Current maintenance consequence |
| --- | --- | --- |
| Smoke command IO: optional stdin, captured/inherited output, three stderr policies, cleanup/handoff, cancellation/deadline | Python request → JSON files/socket → Rust Host bridge → JSON byte arrays → Python outcome | Changes cross request construction, bridge admission and error/evidence reconstruction; Python owns outer helper supervision while Host owns the actual child. |
| Agent calls and receipt checkpoints | Python orchestration → artifact subprocess → ic-agent/Host filesystem owner → returned text or retained bytes | Already-Rust protocol and durable IO are reached through the command transport, including receipt writes. |
| Missing/invalid/stale cache refresh vs cache-only admission | Library loader/classifier → authorized refresh closure → report | CLI separately predicts refresh from mainnet plus `Path::is_file`; existing invalid/stale content can take the live route without that announcement. |
| Candid reply admission | 2 MiB body cap → exact-one-text IDL decoder → unchanged report text; original reply saved first | Smoke selects an unconfigured work/type/header policy while production decoding explicitly bounds it. Single-value strictness is intentional and must survive convergence. |
| Managed cache and explicit export | Confined capability handles → Host bounded IO/publication → family errors; exports retain separate alias/descriptor checks | Separation protects authority and caller-selected output semantics; generic path APIs cannot replace it. |

## Findings and next work

**Verdict: FAIL for the traced refresh-announcement contract.** Source inspection
demonstrates a mismatch between CLI announcement and the actual supported live
refresh path. This does not claim a failed CI run, global cache defect or deployed
failure. The other findings are maintenance opportunity and admission-proof gap.

1. **MEDIUM — refresh policy rediscovery in the CLI.**
   `ic-query-cli/src/progress.rs:61` returns for an existing regular file;
   `nns/inventory.rs:156` and the catalog loader can refresh recoverably invalid
   content. Current node/catalog fixtures explicitly exercise that supported
   replacement. The preflight can also announce before authority refusal.
   Emit existing typed `CacheRefresh` events from the library's authorized
   refresh decision and delete CLI filesystem prediction. Keep source-injected
   builders, silent current library APIs, family classifiers and cache-only
   boundaries. **Disposition: fix next, compatible patch.**
   [Query #28](https://github.com/dragginzgame/ic-query/issues/28).

2. **MEDIUM — redundant smoke transport state.**
   `smoke.py:64` creates request/result files, a cancellation socket and helper
   process per command; `governance_process.rs` translates that protocol into
   Host, while Python reconstructs statuses and separately supervises the bridge.
   The artifact helper already implements agent/receipt work in Rust. A single
   development Rust smoke executable can call those owners directly and delete
   the Python runner, bridge protocol, transport-only tests and retired helper
   surfaces together. Retain ICP orchestration, existing receipts/reply evidence,
   signal handling, five-second TERM grace/reap bound and successful runtime
   handoff. **Disposition: proposed 0.51.0 hard cut; implementation not selected.**
   [Query #29](https://github.com/dragginzgame/ic-query/issues/29).

3. **LOW — smoke Candid work limits are implicit.**
   `governance_artifact.rs:47` uses `IDLDeserialize::new`; locked Candid 0.10.37
   initializes decoding/skipping/type/header limits to `None`. Production
   `candid_decode.rs` explicitly sets these bounds. Configure the smoke decoder
   without losing exact-one-text/trailing-value rejection or reply retention.
   No amplified-payload exploit or execution result is claimed.
   **Disposition: compatible fix, or include in the single Rust smoke owner.**
   [Query #30](https://github.com/dragginzgame/ic-query/issues/30).

## Rehearsals and no-build decisions

Changing smoke cancellation/evidence currently touches Python construction,
Rust Outcome/Request projection and Python error reconstruction. One Rust owner
removes the transport axis; domain IO policies remain, and signal/async cancellation
must be designed and tested rather than assumed to become simpler automatically.
This is a maintenance claim, not an unmeasured speed or net-LOC claim.

Adding a recoverable cache-content classification currently changes library
policy without updating the CLI's existence prediction. Reusing the existing
typed progress contract eliminates the extra decision site without a new CLI
option, report schema or cache validator.

Changing a Candid work bound currently updates production and smoke independently.
Use explicit existing Candid configuration, retaining different cardinality
contracts. No new parser library, decoder framework or ICP JSON/hex route is needed.

Keep NNS/SNS family policies separate, retain focused host features and the
pure/canister boundary, and preserve descriptor confinement, export hard-link
admission and persistent identity storage. No compatibility aliases or deprecated
API shims were found in the searched current source. External “legacy” IC fields
and historical Registry replay are supported wire/authority contracts, not
repository compatibility code to delete by name. No cache reset is proposed.

## Verification and limits

Fresh issue/PR search found no overlapping open Query work before recording these
findings. Read-only source/package checks confirmed the exact versions and primary
Candid configuration implementation. Existing whitespace checking passed. Prior
0.50.4 installer fixtures and released 0.50.3 native CI are supporting history,
not new audit test runs. No full CI, Rust compilation, focused runtime tests,
native macOS execution or live IC smoke was run for these proposals.

Implement the compatible visibility/admission fixes before, or alongside, the
separately selected minor cut. Port supported fixture behavior to the Rust owner;
qualify interruption, descendant-held pipes, failure evidence, startup/handoff,
receipt failures and actual local/mainnet-smoke paths on the required three hosts.
Retire old source/routes in the same change, preserve caller-owned evidence, keep
schema 1 and update Make, ICP callback, CI and docs together. The maintainer owns
release selection and effects.
