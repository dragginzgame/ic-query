# Registered nodes for one subnet

Status: implemented for topology 0.105.2 and CLI 0.105.4 GitHub delivery.

## Decision and authority

Canic needs the registered machines, operators and configured HostOS for a selected
subnet when considering maintenance and a possible TEE-based placement strategy.
`icq topology subnet nodes` reads the selected subnet's
[configuration](subnet-configuration.md), then uses the existing
[node reader](node-inventory.md) for exactly that record's registered IDs.
It performs no global scan, provider/data-center join or hardware attestation.

Each reader uses anonymous mainnet `https://ic0.app`, the built-in trust root and
Registry `rwlgt-iiaaa-aaaaa-aaaaa-cai` `get_value` through published Platform.
All reads request the latest record independently. Query signatures are verified;
returned values are not consensus-certified. Mutation versions can differ and
are not evidence of a common Registry snapshot. `atomic_snapshot` is false.
Neither registration nor successful reads establish live nodes, spare capacity,
running software, usable TEE protection or permission to change authentication.

## Collection and bounds

The library owns collection; the CLI only parses, renders and chooses process
status. Configuration failure stops before node reads. A mismatched configuration
identity is refused. More than 128 members produces a typed `NodeLimit` retaining
the configuration, without truncation or node reads. Empty membership is accepted
as an empty selected record, without proving active subnet membership.

Nodes are read in batches of at most four; each batch finishes before the next
starts. At most one configuration operation and 128 node operations are invoked.
A single 60-second asynchronous deadline includes configuration. Existing leaf
readers retain their 20-second deadlines, 1 MiB HTTP ceiling and protobuf budgets
(64 KiB, 4096 fields, 128 KiB allocation, 1024 items); node chip and HostOS fields
remain limited to 256 bytes each. These bound each retained result; retained
typed failures can also hold bounded supplier diagnostics. Query adds no retries.
Agent's own bounded discovery/transport work remains inside the leaf deadline;
the operation count is not a promise about individual HTTP exchanges.

The timer takes precedence when simultaneously ready with node work. Pending
futures are dropped on the deadline or caller cancellation; no tasks are detached.
Synchronous decoding is bounded separately, so this is not a hard real-time bound.
There is no persistent cache, history, scheduler or concurrency configuration API.

## Typed and JSON contract

`subnet::nodes::fetch` returns a schema-1 `SubnetNodesReport` with the original
configuration and one row per registered ID in canonical principal order.
Completion order does not change row order. Each row has a selected `node_id`
and an outcome tagged by `status`:

| Status | Meaning |
| --- | --- |
| `read` | Matching `NodeReport`, including its original raw values and source evidence |
| `failed` | Leaf reader failed; original `NodeError` remains available in Rust |
| `identity_mismatch` | Returned node ID differs; unexpected ID retained, record refused |
| `overall_deadline` | Started but cancelled at the overall deadline |
| `not_started` | Deadline prevented starting this selected node |

Per-node failure does not abort peers or later batches. `all_node_reads_succeeded`
is true only when every selected row is `read`, including an empty record
vacuously. `overall_deadline_reached` separately records collection cancellation.
No generic completeness or health claim is inferred from either flag.

`started_at_unix_seconds` is local wall-clock time before collection. `elapsed`
and each row's `started_after` / `finished_after` are monotonic durations serialized
as raw `{secs, nanos}`. Unstarted rows have neither offset; cancelled rows have no
finish offset. Successful reports retain their own local Unix observation times.
These are local collection measurements, not certified source times.

JSON projects failures into stable `error.kind` values: `transport_or_verification`,
`read_deadline`, `clock`, `source_identity`, `codec`, `registry_rejected`,
`missing_content`, `missing_operator`, `invalid_operator`, `chip_id_limit` and
`hostos_version_limit`. Registry rejections also retain raw `code`, `reason` and
`key`. Other supplier error internals remain in the typed Rust cause, not a
stringified JSON diagnostic. No inference of missing records from diagnostic
strings is made. Missing chip/HostOS values remain distinct from present-empty.

## CLI and consumer impact

This is an additive command and library module; existing reports retain their
contracts and schema. No existing retained data requires reset. The adapter
now carries output and exit status separately so a partial report can be printed:

- All selected node reads succeeded: report on stdout, exit 0.
- Any failed, mismatched, cancelled or unstarted node: explicit partial report
  on stdout, exit 1, in both text and JSON modes.
- Configuration/admission failure before inventory: no report on stdout,
  diagnostic on stderr, exit 1. Clap argument errors still exit 2.

Consumers must inspect exit status and coverage before using the evidence. A
nonzero exit does not always mean empty stdout for this command. Text shows one
row per node with its full ID, operator/HostOS references, chip-ID presence
(absent, empty or present) and read status. A shared summary and deduplicated
operator/version keys replace repeated record dumps. Version text is escaped;
raw chip IDs, configuration, per-node timing, verification and error evidence
remain in JSON. Raw Registry rejection reasons are not printed as terminal
control text. Operator/version references apply only within that displayed report.
See [usage](../usage/topology.md) and [CLI examples](../../releases/cli/CHANGELOG.md).

## Predecessor review and verification

Read-only inspection used legacy revision
`5b4951b23c2adfcd3c615961f131e766c4b50283`:
[inventory fetch](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/inventory/fetch/mod.rs),
its [regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/inventory/fetch/tests.rs)
and [node inventory source](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/source/nodes.rs).
The [inventory simplification history](https://github.com/dragginzgame/ic-query-legacy/commit/b89d917d5e94f5bd73771076a59347ea29abd328)
removed a redundant provider-count acquisition while retaining the shared relation
reader; the later owned-codec migration is
`0f242c5e467a5e2ec1d4ab28de68678f4cc0bcd8`.

The legacy reader bounded unordered concurrency, retained canonical output order
and distinguished missing relationships from other typed Registry failures. It
acquired a pinned global relation snapshot and aborted on a missing relation.
Here the accepted scope is one subnet and deliberately partial latest-record
observations. Retain bounded concurrency, ordering, identity context and typed
failures; reuse current readers instead of restoring global acquisition, caches,
DTOs or provider joins. No legacy source was copied. Existing leaf tests retain
missing/invalid operator admission; composition tests add failure identity and
raw Registry rejection preservation without string-based reclassification.

Deterministic tests cover exact/excess membership, empty membership, source
ordering, maximum concurrency, reverse completion order, separate evidence,
partial typed failures, mismatched identities, overall time including configuration,
deadline boundary preference, completed/cancelled/unstarted rows and caller
cancellation. CLI tests cover parsing, alphabetical help, partial output/status,
raw JSON and safe terminal presentation.

On 2026-11-14, focused Linux checks passed: eight collector tests, all 22 CLI
unit/process tests (including the new output and command contracts), Clippy for
both affected packages/all targets, Rustdoc with warnings denied, formatting,
snapshot admission and local documentation links. The patch review retained
the existing leaf owners, no added supplier package or generic collection layer,
and no persistent/background machinery. Prior hosted qualification does not
qualify this addition on macOS; that remains for the next delivery gate.

One separate live read of subnet
`tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe`
started at Unix second `1794671958` and took `5.512921372` seconds. All 40
registered members returned matching node reports; node observation times ranged
from `1794671959` to `1794671963`. CLI exit was 0, `all_node_reads_succeeded` was
true, `overall_deadline_reached` was false and `atomic_snapshot` remained false.
This verifies that bounded collection against the selected endpoint worked at
that time; it does not establish ongoing availability or node health.
