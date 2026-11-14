# Selected subnet observation

`icq topology subnet show SUBNET_ID` reports node identities with public-key leaves
returned in that subnet's authenticated state. This lets Canic inspect observed
node composition alongside an application's placement. It is not a network scan,
a complete Registry inventory, a capacity estimate or an uptime measurement.

## Source and claims

Use Platform's published Agent surface, backed by locked Agent 0.49.3's
`fetch_subnet_by_id`. This performs a fresh subnet-scoped read-state request and
verifies its certificate against the built-in mainnet root, including delegation
authority for the selected subnet. No generic CBOR decoder, external type mirror
or new cryptographic implementation belongs in Query.

The Agent requests `/subnet/<id>` and `/canister_ranges/<id>` through
`https://ic0.app/api/v3/subnet/<id>/read_state`. Query projects only the returned
subnet ID and node-key identities. It excludes self-reported subnet keys and
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

Both implemented reports share one internal mainnet transport policy: anonymous
Agent, fixed HTTPS endpoint, built-in trust root, no root-key replacement,
query-signature verification enabled, no TCP retries, a 1 MiB HTTP response-body
ceiling and a 20-second asynchronous deadline. Agent-internal HTTP work shares
that deadline. Its ingress-expiry/certificate-age setting is explicitly 300
seconds; the Agent rejects older certificates but permits future timestamps.
Local observation time is not a certificate timestamp or an independent freshness
proof. Query does not add a retry engine or persistent cache.

The schema-1 subnet report contains the selected subnet, sorted observed node IDs
and source path/time/verification/coverage evidence. At most 1024 nodes are
retained; excess is an error, not truncation. Reject mismatched subnet identity,
duplicate identities and missing node-key evidence. Transport, verification,
deadline and clock failures use the shared typed `ReadError`; subnet-specific
failures remain in `SubnetError`. A failed read does not establish downtime.

These bounds limit transport bytes and Query's projection. Agent owns certificate
decoding and cryptography; an async deadline does not preempt synchronous work.
The node bound applies after the byte-bounded Agent response has been decoded.

## Ownership and qualification

`ic-topology` owns report semantics, admission and evidence. Its private `mainnet`
module shares only transport policy used by both actual reports. Evidence types
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
