# Selected cloud-engine topology

Status: implemented using Platform 0.33.7's published projection from
[Platform #168](https://github.com/dragginzgame/ic-platform/issues/168).
Archive SHA-256: `3e07e0565c70df9222016e93d6f488312176ef5e7036ded60db8b4bc3f61c05f`,
verified against the Cargo registry on 2026-11-15.

## Canic decision and scope

Before considering consolidation or adding engines, Canic needs to identify an
existing engine's subnet, operator and declared members. Select one engine by its
opaque text ID. Read its public control-plane view and preserve the distinction
between the engine record's node list and membership from the canister's Registry
copy. Neither list establishes live nodes, current capacity or maintenance completion.

The surface is `cloud_engine::engine::fetch(EngineId)` and
`icq topology cloud-engine show ENGINE_ID`. Parse `EngineId` from opaque text;
its private representation and lack of deserialization prevent unchecked IDs
from entering the reader. Parsing preserves raw text and enforces byte bounds.
Do not add engine enumeration, signed owner/admin discovery, subscription/billing
analytics, private metadata, operator-role queries or engine mutations in this
slice. Existing `topology subnet` reports can inspect the returned subnet directly.

## Source evidence

On 2026-11-15, re-read mainnet canister `q6cfj-fyaaa-aaaar-qb77q-cai`'s public
`candid:service` metadata through `https://ic0.app`, with anonymous identity and
the mainnet root key. Its SHA-256 still matched the marketplace contract:
`d9f0f383b3dff911d827060f92993ff92de557361df0b828fbec5b1159f68724`.

The selected method is `getEngineViews : (vec text) -> (vec EngineView) query`.
It accepts at most 100 IDs at source; Query will request exactly one. Unknown IDs
are omitted. Reservations never built are also hidden from callers with no role.
An empty successful response therefore means **not returned**, not proof that an
engine never existed. Anonymous reads conceal names, slugs and platform domains;
none of those fields is needed by this report.

A bounded research call to public `listEngineViews` requested one row to obtain
an example ID. A subsequent `getEngineViews` call naming that engine and a
deliberately unknown ID returned only the selected engine. The observed example:

- Engine: `engine-0b56b29a-7ea0-40b8-a963-6c4539ea4414`.
- Subnet: `lwqyp-rlrzo-pkmir-ptnrd-wtafk-wye2h-trdkd-avei4-q6jmy-v742x-2qe`.
- Operator: `3k7kp-7yaaa-aaacn-aaaaa-cai`.
- Four nodes in each list, equal as sets but returned in different orders.
- Embedded Registry copy: empty configured replica-version string, `halted = false`,
  `maximum_state_size = Some(2199023255552)` bytes.
- No deletion timestamp. Anonymous name/slug were empty and domain absent.

These are live source observations, not qualification of a Query implementation
or guarantees that the example remains present. Reproduce the selected read with
installed ICP CLI (this is a source-review command, not an `icq` feature):

```bash
icp canister call q6cfj-fyaaa-aaaar-qb77q-cai getEngineViews \
  '(vec { "engine-0b56b29a-7ea0-40b8-a963-6c4539ea4414" })' \
  --query --network https://ic0.app --root-key mainnet --identity anonymous
```

## Selected external projection

Platform owns these fields and their bounded Candid encode/decode capabilities:

```candid
type EngineSubnet = record {
  node_ids : vec principal;
  replica_version : text;
  halted : bool;
  maximum_state_size : opt nat64;
};
type EngineView = record {
  id : text;
  subnet_id : opt principal;
  node_ids : vec principal;
  deleted_at : opt int64;
  engine_operator_id : opt principal;
  subnet : opt EngineSubnet;
};
service : {
  getEngineViews : (vec text) -> (vec EngineView) query;
};
```

This is a selected response projection, not the complete live service IDL. Skip
the live record's remaining fields under a separate bounded skipped-work budget;
do not retain owner, subscription ID/state, creation time, names, domains or
`all_in_one`. Preserve raw optional presence and full signed/unsigned ranges.
A present but malformed selected optional payload must fail decoding, especially
`subnet`; it must not become `None` under permissive Candid optional fallback.
Unknown content in deliberately skipped fields must not change selected semantics.

The root `node_ids` is the engine record's snapshot: initially populated at
creation, possibly empty for old backfills, and updated by supported membership
changes. It is not a claim about current Registry membership. The nested
`subnet.node_ids` is membership from the canister's Registry copy. Nested `subnet`
can be absent before provisioning, during copy catch-up or when the copied subnet
list lacks the subnet; absence alone cannot distinguish those reasons.
An empty nested replica version is documented for the standard engine version;
do not call it missing software or translate it to a running version. Preserve
optional state-size bytes without substituting a default or claiming free space.

## Query collection and report admission

Validate the requested ID as nonempty UTF-8 text of at most 256 bytes before IO.
Treat it as opaque: no UUID/principal parser, prefix restriction, normalization or
trimming. Send a single-element vector using Platform's encoder, at most 512
encoded bytes. Reuse Query's fixed mainnet/anonymous/verified-query policy:
one query, 1 MiB HTTP response, 20-second leaf deadline, no retry or cache.

Decode limits: 512 KiB bytes, 64 KiB header, 256 types, 2,000,000 work units,
4 MiB logical allocation, and 128 KiB separately bounded skipped work. Query
admits zero or one view. Reject excess rows, a returned ID differing from the
exact request, overlong fields and invalid relationships rather than truncating.
Each node list admits at most 128 distinct nonempty principals. Preserve raw
order in JSON and compare validated sets separately. Empty lists remain valid;
an absent nested subnet is distinct from an observed empty membership.

Present subnet/operator principals must be nonempty. A nested subnet with no
`subnet_id` conflicts with the documented relationship and is refused. Bound
configured replica-version text to 256 UTF-8 bytes, preserving empty/raw text.
Preserve the full optional signed deletion timestamp, including zero/negative
values; its presence is reported rather than filtering the record away.

Schema 1 retains the requested ID, local start/completion times, elapsed
duration, endpoint/canister/method and query-signature/result-certification flags.
The outcome distinguishes `not_returned` from an admitted engine. Retain both
node lists and their source meaning. When the nested subnet exists, emit sorted
`record_only` / `registry_copy_only` differences; otherwise mark comparison
unavailable rather than comparing the record with an invented empty list.

The method exposes no Registry version or copy-observation timestamp. Do not
borrow either from a separate `registryCopyStatus` read or imply that a one-query
response is an atomic IC snapshot. No direct NNS read or fanout is needed in this
first slice. Transport/codec/admission failure emits no success report and exits
1. A successful empty response exits 0 with explicit not-returned semantics.
Missing optional bindings or a membership difference are evidence, not read errors.

CLI text shows one engine/subnet/operator summary and, where present, a
bordered node-list comparison using `ic-host-text`. Escape raw source text and
label configured state and deletion evidence. JSON preserves all selected raw
values. The root and CLI changelogs include the actual observed example:

```bash
icq topology cloud-engine show engine-0b56b29a-7ea0-40b8-a963-6c4539ea4414
icq topology cloud-engine show engine-0b56b29a-7ea0-40b8-a963-6c4539ea4414 --json
```

The JSON `outcome.status` is `not_returned` or `read`. A read retains its selected
facts under `outcome.record`: `record_node_ids`, optional `registry_copy_subnet`,
and `membership.status` (`unavailable` or `compared`). The nested copy uses
`configured_replica_version` and `maximum_state_size_bytes` to keep units and
configured-versus-running meaning explicit. The report never chooses an output
sink; all presentation remains in the CLI.

## Legacy guidance and acceptance

Reviewed frozen legacy `5b4951b23c2adfcd3c615961f131e766c4b50283`:
[`cloud_engine/source.rs`](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/source.rs),
[`build.rs`](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/build.rs),
[`tests.rs`](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/tests.rs)
and [`wire/mod.rs`](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/cloud_engine/wire/mod.rs).
History [`0f242c5`](https://github.com/dragginzgame/ic-query-legacy/commit/0f242c5e467a5e2ec1d4ab28de68678f4cc0bcd8)
moved those calls to Platform-owned codecs but retained local wire declarations;
the current ownership rule requires published external DTOs as well.

Legacy resolves an operator from a subnet, then performs up to four additional
operator-detail reads. Its regressions preserve absent binding, source identity
and noncertified/non-atomic claims. Keep those lessons, but do not restore the
five-call owner/admin/domain report or scan every subnet. Selected public engine
views now supply the narrower relationship directly. No legacy source is copied.

Supplier acceptance needs independently authored Candid fixtures for empty and
populated vectors, omitted unrelated fields/large skipped fields, present/absent
bindings, nested optional failure, signed timestamp extremes, optional zero/full
Nat64 state limits, malformed/trailing/extra-argument frames and every codec bound.
Query acceptance adds pre-IO ID bounds, wrong/duplicate/excess result identities,
128/excess/duplicate node lists, raw ordering versus set comparison, missing versus
empty nested membership, invalid relationship, source assurance, timeout/cancellation,
terminal escaping and local CLI parsing/exit behavior. Source smoke checks do not
replace these regressions or native-host qualification.

## Consumer qualification

On 2026-11-15, local Linux checks passed seven engine-reader tests and the full
selected `cloud_engine` filter (35 tests, including existing pricing, availability,
candidate matching and the matching configuration fixture). CLI checks passed
17 unit and 20 process tests. Affected-package all-target Clippy, warning-denied
topology Rustdoc, formatting, local links and diff checks passed. The request,
selected response, wrong nested optional type and skipped-field frames were
independently assembled from IDL/binary-format facts, rather than copied from
Platform's regression tests. DTO-generated frames separately exercise Query's
admission boundaries and largest supported node lists.

After installing `icq` on PATH, text and JSON selected-engine calls both exited 0.
The example above still returned four engine-record nodes and four Registry-copy
nodes, with empty set differences. JSON completion was observed at Unix second
1794735329 in 0.45 seconds. A deliberately unknown ID produced `not_returned` and
exit 0 at Unix second 1794735335. All calls used the fixed endpoint/canister and
verified query policy above. This is source smoke evidence, not proof of live
nodes, copied-state freshness or qualification of this pending batch on macOS.
