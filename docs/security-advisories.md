# Local advisory acceptance

## RUSTSEC-2026-0387

Accepted on 2026-10-24 at the maintainer's direction to retain upstream IC
versions. Review by **2026-11-07**, and on the next Platform/Agent adoption.
This is a temporary risk acceptance, not a vulnerability fix or a finding that
all library consumers are unaffected. The dependency gate refuses execution from
2026-11-07 UTC until the acceptance is explicitly reviewed or removed.

The selected native paths use Platform 0.23.5 for Query and Host Tools 0.83.0,
reaching Agent 0.49.3 → elliptic-curve 0.13.8
→ crypto-bigint 0.5.5,
including Agent's k256/p256 paths. elliptic-curve requires crypto-bigint 0.5;
the patched crypto-bigint 0.7.6 cannot replace it through a compatible lock
update. Do not fork cryptography or force an incompatible override here.

The native Host and CLI normal/build graphs were rechecked on 2026-10-31 against
the Query 0.78.0 checkout, commit `ab0b4a37`, with the pending 0.79.0 dependency
adoption, Platform 0.15.2, canonical Types 0.15.1, Foundation 0.73.1 and Host
Tools 0.75.0, using locked, offline Cargo tree inspection. Query and Host still use Agent's default anonymous builder, and
Agent's anonymous signer still returns no signature or public key. This refresh
updates dependency and tracking facts; it
does not renew the acceptance or extend its deadline. No fresh cargo-audit or
side-channel qualification is supplied by this graph inspection.

On 2026-11-04, the pending Query/CLI 0.90 release was reviewed against source
`b1fca763` with locked, offline native Query `host` and CLI normal/build trees.
Platform 0.22.0 and Host Tools 0.82.0 still select Agent 0.49.3, elliptic-curve
0.13.8 and crypto-bigint 0.5.5 through the same ECDSA/k256/p256 paths.
Platform's restored upstream certificate identities do not change those paths.
Query and Host still retain Agent's default anonymous builder; the selected
Agent source still returns no signature or public key for anonymous signing.
The existing custom-Agent exposure, accepted scope and 2026-11-07 deadline
remain unchanged. This is a dependency/source review, not a new acceptance or
side-channel qualification; release admission separately runs the current-feed
audit.

On 2026-11-07, the pending 0.93.4 source at `13e12a4f` was reviewed with
Cargo.lock SHA-256 `bcb22487cb5d64cf8dc331eb9d2b5c7350d973d486eb56bfcaeb71da7e94b0e7`.
Locked, offline native graph inspection still reaches Agent 0.49.3,
elliptic-curve 0.13.8 and crypto-bigint 0.5.5. The current RustSec record still
requires crypto-bigint >=0.7.6; the published Agent line requires elliptic-curve
0.13, so no compatible Agent correction is available. Platform 0.23.6, published
during review, retains that Agent requirement. Current source observations
below narrow the Query-owned operation scope, but are not a timing/CPU
qualification or a correction of the retained dependency. This review does not
renew the acceptance or extend its deadline; the expired gate remains active.

Query's built-in clients use `src/agent/mod.rs` in `crates/ic-query` and Host's
`agent_builder`/`EndpointAgent`, retaining Agent's default anonymous identity.
Agent 0.49.3's `AnonymousIdentity::sign` produces no signature or public key.
Query does not load signing keys. Earlier reviews recorded configured-Agent
query injection; current live adapters have private Agent fields and expose
default construction instead. Public certificate helpers accept Agents only to
authenticate certificates and witnesses, without issuing signed requests. These
source observations do not establish a blanket exemption for Agent consumers.

The [advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html) concerns
secret-dependent timing, with exploitability dependent on operations, compiler,
CPU and observation. This source review is not a side-channel qualification.

The local audit ignores only this advisory in addition to its existing exceptions;
other advisories and warnings still fail the gate. Keep
[Platform #50](https://github.com/dragginzgame/ic-platform/issues/50) open. Remove
this exception when a compatible upstream correction eliminates affected versions
from Query's resolved graph, then rerun the current-feed dependency gate and
relevant Agent/MSRV checks. Do not transfer this acceptance to other consumers.

Remediation and affected-consumer evidence are consolidated in Platform #50.
[Query #102](https://github.com/dragginzgame/ic-query/issues/102) retains historical
consumer evidence and is closed as superseded tracking, not as a vulnerability
fix or completed qualification. Query still owns adoption and consumer checks
when a supplying correction is available. The retired Platform #58 coordination
queue imposes no acknowledgement or approval gate.
