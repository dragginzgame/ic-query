# Selected subnet configuration

Canic needs to inspect registered membership, replica-version configuration and
halt flags when planning subnet maintenance or comparing application placement.
The report reads one Registry subnet record, separate from the subnet-certified
observations in `topology subnet show`. `topology subnet configuration` is
implemented using published Platform and included in CLI 0.105.3's GitHub delivery.

## Source contract

Query the existing mainnet Registry `get_value` method for
`subnet_record_<canonical subnet principal>`, omitting a historical version.
The fixed Registry canister is `rwlgt-iiaaa-aaaaa-aaaaa-cai`, through
`https://ic0.app`. A single returned record provides the selected fields without
a network inventory, node/provider join or second certified-state read.

The source is the [SubnetRecord schema](https://github.com/dfinity/ic/blob/4c115dfc3d878cce5cffdb4ee7f34a4671d75628/rs/protobuf/def/registry/subnet/v1/subnet.proto)
and [Registry key constructor](https://github.com/dfinity/ic/blob/4c115dfc3d878cce5cffdb4ee7f34a4671d75628/rs/registry/keys/src/lib.rs),
reviewed at immutable upstream revision `4c115dfc3d878cce5cffdb4ee7f34a4671d75628`.
The required external projection is deliberately limited:

| Protobuf field | Number | Query meaning |
| --- | --- | --- |
| `membership` | 3 | Every node ID listed in this returned record |
| `replica_version_id` | 9 | Raw configured version string, not measured running software |
| `subnet_type` | 15 | Raw signed enum discriminant, including unknown values |
| `is_halted` | 17 | Registry halt configuration, not observed downtime |
| `halt_at_cup_height` | 28 | Configuration to halt at the next catch-up-package height, not proof that the height was reached |

Cloud engines may intentionally have an empty `replica_version_id`: their
version can be selected by a separate `StandardEngineReplicaVersionRecord`.
Preserve the empty string and explicitly mark effective-version resolution as
outside this report. Do not substitute the local Platform version or infer a
running replica version. Fields absent under proto3 scalar semantics retain
their schema defaults; absent content in the get-value envelope remains an error.

## Query report and limits

Schema 1 retains the requested subnet ID, sorted unique `registered_node_ids`,
raw `replica_version_id`, raw `registered_subnet_type`, and both halt flags.
The record key supplies the subnet identity; this record has no embedded subnet
ID to cross-check. An empty membership is preserved as empty record evidence,
not converted into an assertion that the subnet is active or operational.

Evidence includes endpoint, Registry identity/method/key, local completion time,
record mutation version and raw mutation timestamp. Query-signature verification
is true only after Agent success; result consensus certification remains false.
`membership_complete` means every membership entry in this returned record was
retained, not proof of current subnet-list membership, node liveness or a global
Registry snapshot. Other SubnetRecord fields are intentionally unreported.

Reuse the shared 20-second asynchronous deadline and 1 MiB HTTP ceiling. Admit
at most 64 KiB per protobuf envelope/record, 4096 decoded fields, 128 KiB of codec
allocation and 1024 membership entries. Bound the retained version string to
256 UTF-8 bytes. Reject excess rather than truncate; validate each principal and
reject duplicates. Preserve unknown enum values. Quote/escape the version string
in text and keep it raw in JSON. Refuse missing, malformed or chunked content and
preserve typed source errors without printing arbitrary source text.

## Published Platform integration

Platform 0.33.2 supplies the decoder requested in
[Platform #159](https://github.com/dragginzgame/ic-platform/issues/159).
Its stable, non-yanked registry release was verified on 2026-11-14. The archive
SHA-256 is `480c322b9dd62ca6f4f3e9ba80284c6a0d187d9659e3b118af936b814c497431`,
with packaged source revision `f09a0395b60dd8c550c61b70236ecb1e5a934431`.
Its protobuf module SHA-256 is
`92ee23842b853d2a964c1c87cb20e85d90388175a29627ee20aec96c6a1f7c70`.
It exposes a borrowed `SubnetRecord` and `decode_subnet_record` with the existing
explicit limits. Unknown fields, proto3 defaults and signed discriminants remain
owned by Platform. Query adds no external DTO mirror, direct Agent/Prost
dependency or generic protobuf framework.

Initial development used the explicitly authorized local fix at
`661fdc434fe951e2a97fef3f05c44d6596e9b278`; the published decoder retains that
module fingerprint. Registry Platform 0.33.2 and Types 0.20.3 initially replaced both
sibling paths, and the temporary AGENTS exception is retired. No supplier source
was changed by Query. The source transition preserves every other locked package
selection. The subsequent [node integration](node-inventory.md#published-platform-integration)
selects Platform 0.33.3 and its required Types 0.20.4, with unchanged Types source.
The separately adopted Toko Tooling version remains 0.43.1.

Query owns canonical-principal and duplicate admission, version-string bounds,
evidence and report projection. Configuration and list reports share Registry
limits and method selection with the node reader in `mainnet`; existing transport
admission remains with `mainnet`. Report evidence and typed failures stay with
each report. No public query framework or cache is introduced.

## Predecessor lessons and acceptance

Reviewed legacy revision
[`5b4951b`](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283),
its [pinned catalog reader and regressions](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/source/subnet_catalog),
[topology projection regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/projection/subnet_topology/tests.rs)
and [owned-codec migration](https://github.com/dragginzgame/ic-query-legacy/commit/0f242c5e467a5e2ec1d4ab28de68678f4cc0bcd8).
Keep strict identity/duplicate checks and the distinction between requested
snapshot version and record mutation version. The old reader pinned a global
version for multi-record joins and tested missing provider/operator relations;
this single-record report has no such joins and cannot claim their coverage.
Do not restore its caches, routing shards, broad catalog or legacy source traits.

Query acceptance covers deterministic membership, malformed/duplicate/excess
identities, empty membership, empty/cloud-engine version, exact/excess UTF-8 byte
bounds, unknown enum values, both halt flags, raw mutation evidence and typed
failures. CLI acceptance covers help, argument rejection, raw JSON and escaped
text with explicitly configured-versus-observed wording. A live read must retain
source/time/result evidence separately from fixture and native-host qualification.

## Qualification evidence

On 2026-11-14, the registry-selected Platform 0.33.2 / Types 0.20.3 graph passed
32 topology tests, seven presentation tests and eleven CLI process tests on Linux.
Strict Clippy and Rustdoc also passed for both affected packages. Cargo metadata
confirmed one registry identity per supplier and no external path packages.
The published decoder fingerprint matches the earlier local integration; no
product source changes were needed for adoption. The drift review removed the
temporary dependency paths, exception and stale setup claims, preserving report,
transport and CLI ownership. This local evidence does not qualify the batch on macOS.

The earlier 2026-11-14 live read against the local integration for
`tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe` through
`https://ic0.app` completed at local Unix time `1794663473`. It returned 40 members,
configured version `83f29e0803496ba6aaea3d6b43f6c6bebea12dd9`, raw subnet type `2`,
and both halt flags false. Record mutation version was `65525`, with timestamp
`1794207857634545232` Unix nanoseconds. Query-signature verification was true and
result consensus certification false. This is one configuration observation,
not running-version, uptime, native macOS or published-package qualification.
