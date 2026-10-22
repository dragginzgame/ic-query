# Private Agent query boundary

`agent_transport` owns ordinary native query dispatch and structural Agent error
inspection. It forwards caller-supplied bytes, canister and dynamic method names
to the existing configured Agent and returns raw bytes or a private
`QueryCallError` containing the original typed Agent cause.
It does not encode Candid, construct an Agent, authenticate certificates, or add
retry, deadline, polling or update behavior.

`candid_call` retains Candid argument/reply handling. Registry retains its
acquisition-only retry loop, delays, counters and progress events. Its retry
predicate selects connect/timeout failures and HTTP 502/504 from transport facts;
429/503 remain with upstream Agent's existing behavior. ICRC capability policy
interprets structured rejection fields and exact IC0536 codes; it does not infer
unsupported methods from free-form messages. The owned error preserves the original Agent diagnostic via Display and exposes
the original AgentError through std::error::Error::source. No error is converted
to a string at the transport boundary. Family adapters retain their existing
final diagnostic projection.

`QueryFailure` exposes HTTP status, transport connect/timeout flags and borrowed
structured rejection facts, including certified versus uncertified rejection.
Agent response-wait timeout has a separate category: it is not a transport
timeout and does not prove execution failed. No category grants retry authority.
Both selected upstream enums are exhaustively matched; new variants fail the
all-feature build until reviewed. `Other` lists reviewed cases explicitly,
including verification and response-bound errors, with their complete cause
retained. It is not an unknown wildcard. If upstream becomes non-exhaustive,
that is a new mapping decision rather than an automatic catch-all.

CMC, ordinary and certified Registry query requests, Governance reads and dynamic
archive callbacks use the same dispatch function. Certificate/witness validation
and decoding remain with their existing owners after the raw query completes.

## Remaining upstream coupling

This is a private implementation boundary, not an Agent replacement. Query still
uses Agent identities in private transport inputs and Host construction.
Ordinary query-family error adapters now accept the owned error; its source
remains upstream-coupled for diagnostics. Public certificate authentication functions and
`CertificateVerificationError::Invalid` retain their upstream identities.
SNS update/call-and-wait, certified read-state and certificate verification remain
explicit separate Agent operations. Endpoint/response-bound tests may construct
and call Agent directly to qualify those settings. Host continues owning its
existing endpoint binding and builder integration.

There is no new public client abstraction, fork, dependency removal, cryptographic
change or claimed size/performance improvement. Any future public API narrowing
requires its own compatibility decision and actual consumer evidence.
