# Portable topology fixture corpus

`corpus-v1.json` contains 313 frozen frames from Platform revision
[`53efb18`](https://github.com/dragginzgame/ic-platform/tree/53efb18ccd31f4ed7ec6f611488fe27820fbd0ef).
Its SHA-256 is `7613ee62431c7208dfa32ceff52ef6f52f0933b60b1f5f3211e013388692135b`.
`mod.rs` is the same-owner replay helper from that revision, formatted here.
These are test-only inputs, included in the topology package; no runtime or
private sibling dependency is required. Missing input frames fail explicitly.

The supplier's [corpus provenance and regeneration instructions](https://github.com/dragginzgame/ic-platform/blob/53efb18ccd31f4ed7ec6f611488fe27820fbd0ef/qualification/query-hard-cut/portable/README.md)
identify the independently authored schemas and native producer. Regeneration
belongs at Platform, using its locked generator. Review new inputs and byte
changes there before adopting a new corpus identity here. Preserve full-width
JSON integer tokens and the string-encoded floating-point bit patterns.

The transport migration adapts the same-owner [consumer patch](https://github.com/dragginzgame/ic-platform/blob/53efb18ccd31f4ed7ec6f611488fe27820fbd0ef/qualification/query-hard-cut/consumer.patch).
The pinned `../mainnet/root.der` is the mainnet trust key retained from the
previous transport, not discovered from a response; its SHA-256 is
`737ba355e855bd4b61279056603e05501db5e5bad147c6eba7be8c2a13f4b6b3`.

Targeted predecessor review used revision
[`5b4951b`](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283),
its [transport implementation](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/agent_transport/mod.rs),
[regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/agent_transport/tests.rs)
and [ownership history](https://github.com/dragginzgame/ic-query-legacy/commit/978446428b86c7af944e45325933c696e1d34e9f).
No legacy code was copied. Retained invariants are no added retries, exact raw
arguments/results, structured supplier errors and no diagnostic-string inference.
Mainnet tests exercise deadline, result and structured rejection preservation.
Wire regressions replay independent frames. Dynamic method dispatch and unsigned
HTTP fixtures are excluded: Query exposes fixed read-only methods, while Platform
owns HTTP and authentication qualification. Historical passes do not qualify
this adoption.
