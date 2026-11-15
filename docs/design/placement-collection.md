# Selected canister placement collection

Status: implemented locally. The Canic decision is to see which subnets host an
explicitly selected set of bound mainnet canister IDs before considering topology
changes. This is diagnostic routing evidence, not a fleet inventory or deployment
approval. The [Canic consumption boundary](canic-consumption.md) retains ownership
of network selection, application roles and operation-bound placement checks.

## Contract and ownership

`placement::collection::fetch(&[Principal])` admits 1–32 unique IDs before IO.
Empty, excessive and duplicate selections fail rather than being truncated or
silently deduplicated. Selection order is retained. Mainnet is fixed; a Canic
consumer must reject local/custom-network selections before calling Query.
The CLI is `icq topology canister placements CANISTER_ID... [--json]`.

Reuse [single-canister placement](canister-placement.md): each call queries
Registry `get_subnet_for_canister`, preserving its typed error or complete report,
20-second deadline, verified query signature and bounded reply decoder. Reads are
sequential, with one shared 60-second asynchronous deadline and at most 32 logical
lookups. Agent verification work remains inside each leaf budget. No retry,
background task, subnet fanout, discovery, persistence or new transport is added.
Sequential reads keep request pressure bounded; a slow selection can exhaust the
budget before every lookup starts. This is explicit partial coverage, not a promise
that all 32 slow lookups will complete. Synchronous work is bounded separately.

Schema 1 includes local start time, monotonic elapsed time, `atomic_snapshot = false`,
`collection_complete`, `overall_deadline_reached` and ordered `canisters` rows.
Each row retains the selected ID and a tagged outcome:

- `read`: the complete original report, including source and observation time.
- `failed`: original typed Rust error; JSON category and raw Registry rejection
  reason where present, without transport diagnostics being promoted to a schema.
- `identity_mismatch`: unexpected returned canister identity; no usable placement.
- `overall_deadline`: active lookup cancelled by the shared budget.
- `not_started`: shared budget prevented this lookup from starting.

Leaf failures do not stop later reads. Cancellation drops the active future and
starts no further lookup. Only all successful matching reads make the collection
complete; all-failed collections still return their evidence. Initial admission
or clock failure produces no collection. The source supplies no Registry version,
so even a complete collection is not an atomic snapshot. Routing observations do
not establish existence, liveness, controllers, fleet membership or suitability.

The CLI renders bordered canister/subnet/outcome/time rows through `ic-host-text`.
It escapes untrusted diagnostics while JSON retains raw rejection text. Exit 0
means complete acquisition; partial output remains on stdout with exit 1.
Admission failure also exits 1 without a success report; Clap syntax errors exit 2.
Text is presentation only and must not become a consumer parsing interface.

```bash
icq topology canister placements \
  ryjl3-tyaaa-aaaaa-aaaba-cai \
  rwlgt-iiaaa-aaaaa-aaaaa-cai
icq topology canister placements \
  ryjl3-tyaaa-aaaaa-aaaba-cai \
  rwlgt-iiaaa-aaaaa-aaaaa-cai --json
```

These public examples are syntax demonstrations, not inferred Canic bindings.
The library owns collection and errors; the CLI owns argument parsing, output
and exit projection. Single-canister `show` remains unchanged. No generic batch
framework or alternate topology namespace is needed.

## Predecessor guidance and regression coverage

Reviewed the frozen predecessor's
[authority regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/subnet_catalog/tests/authority.rs)
and the previously inspected
[API consolidation history](https://github.com/dragginzgame/ic-query-legacy/commit/b1fca76345efb465d327984d70e9862e1127bbe9).
Its validated routing retains exact catalog provenance and refuses malformed
identity/assurance claims. Retain that principle and test matching selected IDs;
this collection has independent live queries, so it cannot reuse the catalog's
version/digest or consensus claims. No legacy cache, catalog or source is imported.
The current subnet-node collection already demonstrates partial outcomes and
cancellation; reuse its policy pattern without extracting a speculative framework.

Fixtures cover empty/excess/duplicate admission, exact maximum and order, independent
source evidence, leaf failures followed by success, identity mismatch, all-failed
collection, shared deadline and caller cancellation. CLI fixtures cover help,
invalid selection, partial exits, raw JSON and escaped text. No live-network or
new hosted-host qualification is implied by those offline fixtures.

## Local qualification

On 2026-11-15, nine placement tests passed (six collection and three leaf tests),
as did the CLI output test, selection/process test and entity-namespace help test.
Affected-package Clippy, topology Rustdoc, formatting, documentation links and
diff checks passed. Review retained the existing leaf transport/codec owner,
CLI-only rendering and no sibling changes or dependency additions.

An installed-CLI mainnet JSON smoke check of the two example IDs exited 0 with
complete collection in 1.47 seconds, starting at Unix second 1794749777. Both
reported the NNS subnet, with independent completion times 1794749778 and
1794749779, verified query signatures and noncertified results. This confirms
that selected live path only, not fleet ownership, uptime or macOS qualification.
