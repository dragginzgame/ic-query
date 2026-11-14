# Selected node inventory

Canic needs node-level evidence for placement and maintenance planning, including
registered hardware identifiers and configured host software. This bounded
report reads one explicit node's Registry record through `icq topology node show`.
It is included in topology 0.105.1 and CLI 0.105.3's GitHub delivery.

This supplies inputs for later hardware comparisons. It does not answer whether
a machine just came online, has spare capacity, belongs to an active subnet or
supports weaker application authentication. Those claims require additional
observations and an explicit Canic/Auth trust policy.

## Source and selected fields

Use the existing fixed mainnet Registry `get_value` query for
`node_record_<canonical node principal>`, with no requested historical version.
The endpoint is `https://ic0.app`; Registry is `rwlgt-iiaaa-aaaaa-aaaaa-cai`.
The key supplies node identity; the record has no embedded node ID to cross-check.

The [NodeRecord schema](https://github.com/dfinity/ic/blob/904f4dde68006d3cccee4dd3c3ecb4b3798d8790/rs/protobuf/def/registry/node/v1/node.proto)
and [Registry key constructor](https://github.com/dfinity/ic/blob/904f4dde68006d3cccee4dd3c3ecb4b3798d8790/rs/registry/keys/src/lib.rs)
were inspected at revision `904f4dde68006d3cccee4dd3c3ecb4b3798d8790`.
Select only these fields:

| External field | Number | Wire semantics and report meaning |
| --- | --- | --- |
| `node_operator_id` | 15 | Principal bytes identifying the registering operator |
| `chip_id` | 16 | Optional SEV-SNP chip identifier bytes recorded by Registry |
| `hostos_version_id` | 17 | Optional configured HostOS string, not measured running software |

Preserve missing, present-empty and present-nonempty optional fields distinctly.
Do not fill missing values from local configuration. The chip identifier is a
registration claim, not an attestation report, verified measurement or statement
that a TEE is enabled and usable. Absence does not prove absent hardware capability.
Do not infer machine class, performance or cost from Node Rewards fields.

## Published Platform integration

Published Platform 0.33.3 supplies the borrowed `nns_registry_types::protobuf`
`NodeRecord` and `decode_node_record` for these three fields, using existing
explicit limits and the get-value envelope. The decoder owns
protobuf defaults, optional presence, UTF-8 validation, duplicate singular-field
refusal and bounded unknown-field skipping. It authenticates nothing and must
not classify hardware or resolve operator/provider relations.

The stable, non-yanked release was verified on 2026-11-14. Its archive SHA-256 is
`74c6e194ce3a80487320edef0041e286fd89d6d96e8a9f641811c7e38b856e51`, with packaged
revision `9035c3f73a9bd8e866737418daa3b741f14e3fad`. It requires Types 0.20.4;
that release is non-yanked and its source matches 0.20.3 byte-for-byte. Query
selects the same registry Types identity for its report and Agent boundaries.

Supplier wire qualification remains with
[#160 — Add bounded Registry NodeRecord decoding](https://github.com/dragginzgame/ic-platform/issues/160).
Query adds consumer projection regressions, no protobuf implementation, external
DTO mirror or legacy Host Tools protocol owner. No sibling dependency or supplier
source change is introduced. The borrowed decoder allocates no collections;
Query owns the independent retained chip/string budgets below.

## Query projection and bounds

Schema 1 retains `node_id`, validated `node_operator_id`, raw
`chip_id: Option<Vec<u8>>` and `hostos_version_id: Option<String>`. JSON preserves
chip bytes as an array, including null versus empty, and the raw version string.
Text renders bytes as hexadecimal (`0x` for present-empty) and quotes/escapes the
version string. Missing optional fields print `absent`. Both are labelled as
Registry data, with attestation unverified.

Require a nonempty valid operator principal; default-empty operator data is a
typed missing-operator error, not the management principal. Preserve empty
optional values without promoting them to useful evidence. Bound chip bytes and
version UTF-8 bytes independently to 256 each. Reuse the shared mainnet deadline,
HTTP ceiling and Registry protobuf byte/field budgets, now shared with subnet
readers under `mainnet`. Refuse missing, malformed,
chunked or excessive content without truncation or retry. One record read suffices.

Evidence retains endpoint, Registry identity/method/key, local completion time,
record mutation version and timestamp. Query-signature verification remains
distinct from consensus certification. Include `tee_attestation_verified: false`
as an explicit limit. A missing record or failed read does not prove node downtime.

Do not join NodeOperator, NodeProvider, data-center or subnet records, resolve
effective HostOS software, add caches or scan the network. Add such work only for
a concrete consumer decision and a bounded, appropriately versioned source contract.

## Predecessor review and consumer acceptance

Reviewed legacy revision
[`5b4951b`](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283),
its [node projection](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/projection/node.rs),
[relation regression](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/tests/inventory/node.rs)
and [inventory ownership history](https://github.com/dragginzgame/ic-query-legacy/commit/b89d917d5e94f5bd73771076a59347ea29abd328).
The old projection joined nodes to subnets, operators, providers and data centers,
requiring assignment before producing a row. Preserve identity and relation
integrity lessons, but do not require assignment for this selected-record read
or infer provider/location from an operator ID. No legacy source is copied.

Query acceptance covers missing/invalid operator identity, optional presence,
exact/excess byte limits, raw bytes/version JSON, truthful attestation/software
claims, mutation evidence and typed failures. CLI acceptance covers local help,
invalid IDs and escaped text. Copyable examples with a real public node ID are in
the root quick start and CLI ledger. Live smoke evidence remains separate
from fixture, published-package and native-host qualification.

## Qualification evidence

On 2026-11-14, the published Platform 0.33.3 / Types 0.20.4 graph passed
37 topology tests, eight presentation tests and eleven CLI process tests on Linux.
Five new node projection tests cover raw values, all optional-presence combinations,
identity admission, exact/excess byte limits and typed failures. Strict Clippy and
Rustdoc also passed for both affected packages. This is not macOS qualification.

One live read through `https://ic0.app` for node
`nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae` completed at
local Unix time `1794670789`. It returned operator
`3bohy-zygmq-6novb-wm65r-3k3y2-deh42-7emld-zhqo7-xl2oa-iextm-fae`, absent chip ID
and configured HostOS `4a0dbb9251df6da0b65ab7a4d35e9a9871f58510`.
Record mutation version was `64882`, timestamp `1792070087547961300` Unix
nanoseconds. Query-signature verification succeeded; result certification and
TEE-attestation verification remained false. This is one Registry observation,
not evidence that hardware lacks a TEE or that the configured software is running.

The drift review retained report projection in the library and hexadecimal/text
formatting in the CLI. Only the existing Registry limits/method constants moved
to their shared mainnet owner; no query framework, scan, cache or duplicated
decoding policy was introduced.
