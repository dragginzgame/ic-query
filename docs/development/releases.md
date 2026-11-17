# Releasing Query

The shared Toko Tooling runners own frozen source capture, version transactions,
commits, annotated tags, atomic pushes and publication attempt records. Query's
`scripts/release.sh` owns group selection, the complete local gate, source-bound
receipts and its fixed direct-delivery policy. Each of topology, sns-migration,
query and cli must be selected explicitly; an absent group cannot publish all packages.

The family-name cutover maps `topology` to `ic-query-topology` and
`sns-migration` to `ic-query-sns`. The group names, ledger paths and prior tags
remain stable; old registry packages are not aliases and will not be republished
under the new names. The first renamed library releases used minors
(topology 0.110.0, SNS 0.105.0); the unchanged CLI contract advanced to 0.105.15.
Release both renamed libraries before the CLI, then update downstream package
names and Rust imports. Retained report evidence requires no reset.
The new umbrella follows the specialist libraries: `query` advances the published
legacy 0.103.1 baseline to 0.104.0 with an explicit minor release. That cutover
used `topology,sns-migration,query` with the minor runner, reconciliation, then
`cli` with the patch runner. The umbrella's own ledger defines legacy
feature/module retirement; no automatic legacy-data migration is claimed.

Commit the reviewed implementation and pending notes first. Manifests retain the
last delivered group versions until the runner advances them. From a clean `main`
checkout with GitHub push and crates.io credentials available, use:

```bash
make release-patch RELEASE_GROUPS=topology,sns-migration,query,cli
make release-batch-reconcile BATCH=batch-XXXXXXXX
```

The batch identifiers are printed by the runner; use the actual identifiers from
the run. These commands push to `origin/main` and publish the selected group.
Topology and migration precede CLI so its dependencies are available. The initial
migration publication advances the unpublished 0.104.0 baseline to 0.104.1 using
the standard patch runner. Choose patch/minor/
major from the selected package's compatibility impact; all use the same gate.
Mixed increments use separate reconciled batches. Independent groups with the
same increment can use an explicit supplier-first `RELEASE_GROUPS` roster.

Setup fetches the locked graph, prepares the declared host/Rust developer tools
and metadata editor `ic-build` 0.73.0, then checks them offline. Recovery only
admits the saved selections. The complete gate runs sequentially through
`make check`: tooling, links, formatting, strict Clippy, Rustdoc and tests. No
partial group validation or cross-version evidence reuse is claimed. Each group
runs the complete gate. Metadata checks bind the passed source and exact payload;
publication separately requires a clean source at its annotated group tag and
uses Cargo's normal package verification. Hosted CI remains asynchronous.

The authenticated editor is selected through the shared installer, with no
per-shell path setup. Release metadata includes the group ledger, owned Cargo
manifests and lockfile; the CLI also finalizes the root quick-start heading.
Unrelated dependency updates are excluded.

Use `make release-batch-status BATCH=...` to inspect an attempt and
`make release-batch-resume BATCH=...` for its explicit recovery. Never start a new
upload after an uncertain result without checking registry state and the retained
publication intent. Batch storage lives under the repository's ignored `target/`
with compact Git recovery records; normal release does not clean it. Reconciliation
only fast-forwards a clean normal checkout. See the pinned
[batch contract](https://github.com/dragginzgame/toko-tooling/blob/626da7cd9626cc2e152594862a2d82ea82ef51da/docs/release-batches.md).

Linux adapter qualification on 2026-11-16 exercised both actual group metadata
hooks in an isolated repository: untracked-source refusal, failed validation
without a success receipt, successful receipt binding, preparation, staged and
committed payload admission, missing-tag refusal and annotated-tag admission.
The fixture substituted a deterministic product gate to test orchestration;
it did not push or upload and is not product qualification. Each actual release
still runs Query's complete gate and Cargo's package verification. Hosted macOS
execution remains separate evidence.
