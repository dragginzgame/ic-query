# Authenticated subnet state-time progress

Status: implemented using published replacement Platform 0.34.7 under the
[development supplier selection](../development/setup.md#certificate-time-supplier).
The supplier capability is delivered in
[Platform New #1 — Authenticated subnet certificate time](https://github.com/dragginzgame/ic-platform-new/issues/1).
The hard cut removes legacy Platform and Agent from Query; the supplier's
separate [Platform #4 — Agent advisory](https://github.com/dragginzgame/ic-platform/issues/4)
still has obligations for other consumers.

## Decision and source

Canic needs state-progress evidence during quiet application traffic. The command
`icq topology subnet progress SUBNET_ID --interval-seconds 10 [--json]` and library
`subnet::progress::fetch` compare two authenticated signing-certificate timestamps.
Advanced, unchanged and regressed are exact orderings, not healthy/stalled/reset
labels. They do not establish continuous uptime, running version or maintenance
completion. Existing activity and maintenance commands are unchanged.

Use fixed mainnet Platform native reads and `/api/v3/subnet/<id>/read_state`.
The request selects `/subnet/<id>` and `/time`. Platform returns authenticated
original certificate bytes; its strict decoder and borrowed projection recheck
signatures, keys, exact subnet authority, tree limits and freshness at completion.
The report does not retain those bytes. Platform owns outer decoding under the
1 MiB message ceiling and bounds subsequent traversal and delegation decoding.
Query adds no CBOR, LEB128 or cryptographic codec.

The built-in mainnet root key is paired with the fixed NNS root subnet
`tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe`.
This trust input is independent of request and response. Root/no-delegation
certificates for any other selected subnet are refused. Query, Protocol and
Platform share one Types identity, without a conversion bridge.

## Bounds and freshness

- Two sequential reads, one wait of 1–60 seconds (CLI default 10), validated before
  reader construction. The wait begins after the first read completes.
- Existing 20-second asynchronous bound per read; at most 100 asynchronous seconds
  plus bounded verification and scheduling. No retry, fanout, persistent monitor
  or detached task. Cancellation starts no subsequent work.
- Combined visible-tree allowance 4096 nodes, root-inclusive depth 64, embedded
  delegation at most 128 KiB. Canister ranges are unused for subnet authority.
- At each completed read capture local Unix nanoseconds. Independently require
  signing and delegation times within inclusive 300-second past/60-second future
  windows. A legitimately older delegation is refused, not silently admitted;
  this conservative initial policy may reduce availability. No fallback expands
  the window. Local clock failures/overflow refuse the observation.

## Evidence and failures

Schema 1 retains subnet, requested interval, total monotonic duration, the first
observation and the second outcome. Each observation includes raw signing and
optional delegation times, local verification reference, endpoint/path, explicit
freshness limits and source assurances. Original bytes are explicitly not retained.
The comparison ignores local-clock ordering and uses exact u64 arithmetic only
when the second certified time is larger. Decrease never underflows.

A first-read failure returns a typed error without attempting the second read.
A second-read failure retains the first observation and typed cause, with stable
JSON categories; partial stdout exits 1. Two successful observations exit 0 even
when equal or regressed. Wrong identity/unverified results are refused without
promoting them to evidence. Invalid CLI syntax exits 2.

## Qualification and legacy guidance

Consumer tests cover the published certificate projection using supplier
PocketIC frames, exact borrowing, separate time retention, wrong authority and
corrupted signatures. Controlled-clock tests cover interval boundaries, acquisition
order, cancellation during reads/wait, failures, partial JSON, full-width arithmetic
and local-clock reversal. Supplier cryptographic/decoder proofs remain upstream;
these tests do not substitute for them or establish macOS/mainnet qualification.

Local Linux checks on 2026-11-16 passed all 133 topology tests, CLI unit/process
checks, Clippy and Rustdoc. Live one-second reads through the installed CLI
returned complete root-subnet JSON with unchanged time and complete delegated
subnet text with advancing time. These are smoke observations, not continuous
availability evidence. The supplier product source remained at the inspected
commit; unrelated ongoing supplier qualification edits were left untouched.

The [maintenance source assessment](subnet-maintenance.md#state-time-source-assessment)
records the reviewed legacy Dashboard admission and authority lessons. Preserve
unknown/partial evidence and source distinctions; import no Dashboard global
inventory, reward fields or cache. This implementation reuses mainnet transport
and the supplier projection without creating a universal sampling framework.
