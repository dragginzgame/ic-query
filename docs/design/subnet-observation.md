# Selected subnet observation

`icq topology subnet show SUBNET_ID` reports subnet type and node identities with
public-key leaves returned in that subnet's authenticated state. This lets Canic
inspect subnet class and node composition alongside an application's placement.
It is not a network scan, a complete Registry inventory, a capacity estimate or
an uptime measurement.

## Source and claims

Use Platform's published Agent surface, backed by locked Agent 0.49.3's
`fetch_subnet_by_id`. This performs a fresh subnet-scoped read-state request and
verifies its certificate against the built-in mainnet root, including delegation
authority for the selected subnet. No generic CBOR decoder, external type mirror
or new cryptographic implementation belongs in Query.

The Agent requests `/subnet/<id>` and `/canister_ranges/<id>` through
`https://ic0.app/api/v3/subnet/<id>/read_state`. Query projects only the returned
subnet ID, optional subnet type and node-key identities. It excludes subnet keys and
canister ranges: a subnet's own attestation is not NNS authorization to route a
canister to that subnet. The existing Registry placement report remains separate.
See the supplier's
[fetch contract](https://docs.rs/ic-agent/0.49.3/ic_agent/struct.Agent.html#method.fetch_subnet_by_id).

The Agent projection does not expose a proof that its enumerated nodes constitute
complete Registry membership. The report therefore sets `membership_complete`
to false and leaves `registry_version` null. Certificate verification establishes
the authenticated source observation, not node liveness, health, geographic
diversity, TEE availability, capacity or Registry agreement. No result makes
those claims implicitly through the word "subnet".

## Bounds and report

The implemented reports share one internal mainnet transport policy: anonymous
Agent, fixed HTTPS endpoint, built-in trust root, no root-key replacement,
query-signature verification enabled, no TCP retries, a 1 MiB HTTP response-body
ceiling and a 20-second asynchronous deadline. Agent-internal HTTP work shares
that deadline. Its ingress-expiry/certificate-age setting is explicitly 300
seconds; the Agent rejects older certificates but permits future timestamps.
Local observation time is not a certificate timestamp or an independent freshness
proof. Query does not add a retry engine or persistent cache.

The schema-1 subnet report contains the selected subnet, observed type, sorted
node IDs and source path/time/verification/coverage evidence. At most 1024 nodes are
retained; excess is an error, not truncation. Reject mismatched subnet identity,
duplicate identities and missing node-key evidence. Transport, verification,
deadline and clock failures use the shared typed `ReadError`; subnet-specific
failures remain in `SubnetError`. A failed read does not establish downtime.

These bounds limit transport bytes and Query's projection. Agent owns certificate
decoding and cryptography; an async deadline does not preempt synchronous work.
The node bound applies after the byte-bounded Agent response has been decoded.

## Ownership and qualification

`ic-topology` owns report semantics, admission and evidence. Its private `mainnet`
module shares only transport policy used by actual reports. Evidence types
remain distinct because query signatures and certified state support different
claims. No generic query framework or cross-crate helper is needed.

The CLI's `topology` adapter owns its command tree and dispatch. Pure presentation
functions format reports; the entrypoint alone owns runtime setup and process IO.
Text displays evidence from the report instead of hardcoding verification claims.
JSON retains canonical principals and raw values. CLI group changelog examples cover
both text and JSON operations.

Offline tests cover source identity, deterministic order, empty/duplicate
observations, the exact node bound, over-limit refusal, raw JSON evidence,
transport/deadline failures, pure rendering and argument/help behavior. These
fixtures do not prove cryptographic validity or live availability. Live read-state
observations are separate smoke checks; hosted macOS qualification remains pending.

The unpublished placement API now wraps shared read failures in
`PlacementError::Read(ReadError)`; its schema-1 JSON is unchanged. The in-workspace
CLI and tests migrate together. No persisted format reset or legacy CLI
compatibility adapter is introduced by this internal ownership cleanup.

## Observed subnet type

Canic needs to distinguish subnet classes when comparing application placement
and cloud-engine options. The report extends the existing observation using the
optional `/subnet/<id>/type` leaf already decoded by the same Agent fetch. This adds no
request or Registry join. The certificate authenticates the selected subnet's
statement; it does not establish independent Registry agreement, deployment
permission, pricing, available capacity or TEE capability.

The schema-1 field is `observed_subnet_type: Option<String>`. Preserve
`application`, `verified_application`, `system` and `cloud_engine` distinctly;
retain unrecognized source strings without normalization. `null` means the Agent
proved the leaf absent, not that the subnet has an unknown type. Pruned, malformed
or unverifiable evidence remains a read failure. Limit a retained type to 128
UTF-8 bytes and reject excess with a typed subnet error; do not truncate it.
Text presentation quotes and escapes the string so unknown values cannot emit
terminal control characters; JSON retains the raw string.

Published Platform 0.33.1 supplies `ic_platform::agent::SubnetType`, preserving
the upstream identity. Query matches its variants directly, including
`Unknown(String)`; it adds no external type mirror or direct Agent dependency.
The adopted non-yanked registry archive has SHA-256
`4d780999e8b7e89cf9f3f78c979daaaf0299127451d904636867c61b56cd233c`
and identifies release commit `512063b53dd30647ae1887372f8507ae31bfdd79`.

The legacy review used revision
[`5b4951b`](https://github.com/dragginzgame/ic-query-legacy/tree/5b4951b23c2adfcd3c615961f131e766c4b50283),
its [classification mapping](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/subnet_catalog/model/classification/kind.rs),
the [mapping consolidation](https://github.com/dragginzgame/ic-query-legacy/commit/c34c4881b8993e8d8b1d6ccb5480363705c836cf)
and [topology projection regressions](https://github.com/dragginzgame/ic-query-legacy/blob/5b4951b23c2adfcd3c615961f131e766c4b50283/crates/ic-query/src/ic_registry/projection/subnet_topology/tests.rs).
That Registry-based implementation grouped verified application with application
and discarded unknown discriminants. Keep its explicit classification and strict
identity-validation lessons, but preserve these distinctions in the new source
contract. Do not carry over default charging inference or Registry completeness
claims to a subnet's certified observation.

Offline qualification covers each known type, absent versus unknown (including an
empty unknown string), exact and exceeded UTF-8 byte bounds, raw JSON retention
and escaped text. Existing identity, node and transport regressions remain in
force. Adding a public Rust struct field breaks downstream struct literals, so
the topology delivery requires a minor version (0.105.0), with the CLI and
fixtures migrated together. JSON remains schema 1 with an additive field; no
persisted evidence reset is selected. The field first ships in the 0.105.0 library
and 0.105.2 CLI; the earlier 0.104.1 library and 0.105.1 CLI do not contain it.

On 2026-11-14, a Linux live smoke read through `https://ic0.app` for subnet
`tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe` returned
`observed_subnet_type: "system"`, 40 observed node identities and verified state
certificate evidence. Local completion time was `1794661134` Unix seconds;
membership completeness remained false and Registry version remained null.
This is one successful observation, not native macOS qualification or uptime
evidence. The patch's 15 topology tests, four presentation tests and eight CLI
process tests passed, along with strict Clippy, Rustdoc and formatting checks.
