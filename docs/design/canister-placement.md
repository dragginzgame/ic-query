# Canister placement report

The first topology slice answers: which subnet does the mainnet NNS Registry
currently associate with one explicitly selected application canister? Canic can
use this observation when comparing an application's existing placement.

This is a routing observation, not proof that the canister exists, is running,
has capacity or should be moved. It is not a complete subnet/node inventory.

## Source and boundary

Use published Platform 0.32.3's `agent` and `nns-registry-types` capabilities,
including its bounded owned codec. The supplier exposes
`GetSubnetForCanisterRequest` and `GetSubnetForCanisterResponse`; its Registry
surface does not yet supply full node/subnet inventory records. Those external
contracts must be added at Platform before Query can consume them.

- Endpoint: `https://ic0.app`, mainnet only in this first slice.
- Canister: `rwlgt-iiaaa-aaaaa-aaaaa-cai` (NNS Registry).
- Query method: `get_subnet_for_canister`, with `principal = Some(selected)`.
- Anonymous transport, built-in mainnet trust root, query signature verification
  enabled. Never fetch a replacement root key or disable verification.
- Query signatures authenticate the query response; they do not make its result
  a consensus-certified Registry snapshot. Report that distinction explicitly.

The source method is documented in the
[Registry interface](https://github.com/dfinity/ic/blob/master/rs/registry/canister/canister/registry.did).
The adopted DTO implementation is from Platform commit
[`c90ce9f`](https://github.com/dragginzgame/ic-platform/blob/c90ce9fce0948da0c05e6f5ac333c520fa46d801/crates/ic-platform/src/nns_registry_types/mod.rs).

## Contract

Return a schema-1 typed report with the selected canister, returned subnet,
mainnet endpoint, source canister/method, observation time in Unix seconds,
query-signature verification and explicit absence of result certification and
Registry-version evidence. Completeness applies only to this single lookup.

The library owns source admission, limits, decoding and typed failures. It never
prints. The CLI owns arguments, text/JSON presentation and stdout/stderr. JSON
preserves canonical principal text and raw numeric values. Do not add persistence,
collection, SNS or NNS governance to this operation.

Use a 20-second overall asynchronous query deadline, a 1 MiB HTTP response-body
ceiling (including Agent verification reads), a 64 KiB decoded reply-byte ceiling,
and explicit codec header/type/work/allocation budgets. No Query retry loop;
disable Agent TCP retries. The shared internal mainnet module owns these settings
for both implemented reports, with an explicit 300-second Agent ingress expiry.
Agent may perform its own bounded-by-deadline HTTP
retry or signature-key discovery work. Bound synchronous decoding separately.

Fail with distinct transport, deadline, codec, Registry rejection, missing subnet
or clock errors. Preserve supplier errors and Registry text; do not infer
not-found or downtime by parsing diagnostic strings. Never emit a partial report.

Transport, deadline and clock failures are carried by `PlacementError::Read` using
the shared `ReadError`. This unpublished Rust API cleanup preserves schema-1 JSON;
see the [subnet contract](subnet-observation.md) for the shared policy and its limits.

## CLI and qualification

Implement `icq topology canister show CANISTER_ID`, with `--json` for machine output.
CLI commands name the inspected entity (`canister` or `subnet`); this canister
report currently supplies routing placement only.
Bare namespaces and explicit help are local and successful; missing or invalid
leaf arguments fail before network access. Report success on stdout, failures on
stderr with a failing status.

Qualify source-response projection, malformed and over-budget replies, empty
subnet and Registry rejection, exact JSON values and CLI argument/help contracts.
Fixtures do not prove live availability or cryptographic verification. Keep a
live mainnet smoke observation separate from repeatable offline tests.

This is a new `ic-topology` API and new `icq topology` command. It does not restore
legacy `icq nns ...` syntax or retire legacy caller and retained-data obligations.
