# Selected-node registration changes

Canic needs to notice changes in registered machine identifiers and configured
host software before investigating placement or maintenance implications. Compare
two observations of one explicit node; do not infer that a machine came online,
that its configured version is running, or that a TEE became usable.

## Contract

`node::changes::compare` accepts two owned `NodeReport` values without IO.
Schema 1 retains both original reports and evidence, with independent flags for
operator, raw optional chip ID and raw optional HostOS changes. Absent,
present-empty and present-nonempty values remain distinct. A newer record version
with unchanged selected fields is valid; no selected changes is a successful
comparison, not a claim about intermediate events or unselected fields.

Refuse different node IDs, unsupported schemas, empty operator identities,
chip/HostOS values above the existing 256-byte limits, source canister/method/key
outside the selected mainnet Registry node-record contract, a regressing record
version, or different selected content at the same record version. Retain both
inputs on refusal. Endpoint and assurance flags are retained without upgrading
them; comparison does not reauthenticate supplied evidence. Local observation
times may regress and are not used as Registry event ordering.

`node::changes::fetch` and `icq topology node changes` use the existing node
reader twice, waiting 1–60 seconds after the first read finishes (CLI default 10).
Each read retains its 20-second asynchronous deadline and current HTTP, codec
and field limits. Invalid waits refuse before IO. Initial failure stops sampling;
second-read failure retains the first observation and original typed cause.
Cancellation starts no later work. No network scan, cache, persistent history,
retry loop or background task is added.

Text shows only changed fields, with escaped HostOS strings and chip presence/
byte counts rather than long hex dumps. A changed chip with the same byte count
is still explicitly labelled changed. JSON retains the full raw before/after
values and evidence. Successful comparisons exit 0, whether or not anything
changed. Read/comparison failure exits 1 with no success JSON; invalid CLI syntax
exits 2. This additive API/command replaces no saved contract; schemas remain 1.

## Predecessor review and acceptance

Read-only review used legacy revision `5b4951b23c2adfcd3c615961f131e766c4b50283`:
[node report model](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/nns/node/report/model/report.rs),
[report regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/nns/node/report/tests.rs)
and [relation regression](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/tests/inventory/node.rs).
History [b89d917](https://github.com/dragginzgame/ic-query-legacy/commit/b89d917d5e94f5bd73771076a59347ea29abd328)
narrowed inventory and source ownership. That report retained Registry provenance
and admitted source identities, but did not contain chip/HostOS comparisons.
Reuse those invariants and the current bounded node reader; restore neither its
global inventory/cache machinery nor rewards-derived hardware interpretations.
No legacy source or third-party decoder implementation is copied.

Acceptance covers every optional-presence transition, raw same-length chip
changes, unchanged fields at newer versions, each refusal retaining both inputs,
raw evidence/assurance retention, bounds, exact sampling order, source failures
and cancellation. CLI checks cover help, interval admission, output status and
escaped compact text. Live checks and hosted native qualification remain distinct
from deterministic tests.

## Qualification evidence

On 2026-11-14, seven new comparison/sampling tests passed locally on Linux
x86-64, alongside all 13 CLI unit tests and 15 CLI process tests. Affected
Clippy, Rustdoc, formatting, snapshot admission and local-link checks passed.
Drift review retained report/collection policy in the library and presentation
in the CLI, reused the node reader's field limits, and shared the repeated CLI
interval argument definition across existing two-observation commands. It added
no generic comparison framework or external protocol dependency.

The installed `icq` was refreshed on PATH. A separate read-only one-second
mainnet sample for node
`nuass-yihti-wszdr-r7qze-32dgz-osovf-pracd-hg5tt-mwlhf-rvuyq-4ae` exited 0,
retaining mutation version `64882` in both observations at local Unix seconds
`1794678131` and `1794678133`, with all three change flags false. This is a
successful point check, not continuous availability or new macOS qualification.
