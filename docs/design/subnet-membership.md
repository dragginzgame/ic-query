# Registered membership versus observed node keys

Canic needs to inspect whether a subnet's registered members appear in its
certified node-key observation during maintenance or a membership transition.
`icq topology subnet membership SUBNET_ID` composes the existing
[configuration](subnet-configuration.md) and [observation](subnet-observation.md)
readers for one explicit subnet. It does not scan the network or infer node health.

Read configuration first, then certified subnet state, once each without retries.
Each read retains its existing admission, deadline and source-specific evidence.
The configuration reader admits at most 1024 members; the observation reader
admits at most 1024 node identities. Both already validate, sort and reject
duplicates, so set differences can use the admitted sorted lists directly.
No new supplier capability, cache, public source trait or shared query framework
is needed. Both readers use published Platform; no local supplier
exception or sibling checkout is required.

Schema 1 retains both complete reports, `atomic_snapshot: false`, and two sorted
sets: `registered_without_observed_key` and `observed_key_without_registration`.
The library checks that both reports identify the requested subnet before
comparison. The CLI owns pure rendering and preserves raw nested reports in JSON.

These are differences between two separately acquired observations. Registry
query signatures do not consensus-certify its content; the subnet's node-key
projection does not establish complete current Registry membership. The reads
have no shared Registry version, certified timestamp or atomicity guarantee.
Local timestamps may even move backwards. Empty differences establish equality
of the returned sets only, not live nodes or a completed maintenance operation.
Differences may reflect transitions, source age or projection coverage; do not
label them as failed nodes, unauthorized nodes or a verified topology fault.

First-read failure returns its typed cause without polling the subnet reader.
Second-read failure retains the successful configuration in the library error.
Identity disagreement returns a typed failure instead of combining subnets.
The CLI preserves its normal nonzero/stderr failure contract, with no success
JSON or automatically retried work. Cancellation starts no background task.

## Predecessor review and qualification

Reviewed legacy revision
[`5b4951b`](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283),
its [topology projection and regressions](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/projection/subnet_topology),
[authority tests](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/subnet_catalog/tests/authority.rs)
and the previously reviewed [owned-codec migration](https://github.com/dragginzgame/ic-query-legacy/commit/0f242c5e467a5e2ec1d4ab28de68678f4cc0bcd8).
Retain identity validation, deterministic membership and source-assurance
boundaries. Unlike the old globally pinned Registry joins, this comparison has
two different source contracts and cannot claim their atomicity or completeness.
Do not restore catalog seals, caches, multi-endpoint agreement or provider joins.

Offline acceptance covers matching, disjoint, overlapping and empty registered
sets, distinct source evidence, wall-clock reversal, wrong-subnet refusal,
first/second failure and bounded retained differences. Existing reader tests
continue to own invalid/duplicate/over-limit identities. CLI checks cover local
help, rejected arguments, truthful text and raw JSON retention. Live reads provide
separate smoke evidence, never a continuous uptime or native-host guarantee.

## Qualification result

The registry-selected Platform 0.33.2 graph passed all 32 topology tests, including
the six membership tests, plus seven presentation and eleven CLI process tests
on Linux. Strict Clippy and Rustdoc passed for the affected packages. The
[configuration evidence](subnet-configuration.md#qualification-evidence) records
supplier admission and scope. The patch review found no new transport policy,
cache, public source abstraction or duplication of reader admission.

On 2026-11-14, the earlier local-integration live comparison through `https://ic0.app` for subnet
`tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe` returned
40 registered members and 40 observed node-key identities with both difference
sets empty. Registry acquisition completed at local Unix time `1794663810`,
with record mutation version `65525` and timestamp `1794207857634545232` Unix
nanoseconds. Subnet acquisition completed at `1794663811`. Registry query-signature
verification and subnet state-certificate verification succeeded; Registry result
certification remained false, observed membership completeness remained false,
and `atomic_snapshot` remained false. This is equality of two returned sets,
not proof of live nodes, synchronized source state or completed maintenance.
