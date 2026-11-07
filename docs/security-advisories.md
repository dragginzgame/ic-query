# Local advisory acceptance

## RUSTSEC-2026-0387

Accepted on 2026-10-24 at the maintainer's direction to retain upstream IC
versions; renewed with explicit maintainer approval on 2026-11-07 for Query
only. Review by **2026-11-21**, and on the next Platform/Agent adoption.
This is a temporary risk acceptance, not a vulnerability fix or a finding that
all library consumers are unaffected. The dependency gate refuses execution from
2026-11-21 UTC until the acceptance is explicitly reviewed or removed.

The selected native paths use one Platform 0.24.0 facade for Query,
Host Tools 0.85.0 and Testkit 0.92.0,
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
qualification or a correction of the retained dependency. The selected lockfile
subsequently advanced only Platform to 0.23.6; it retains the same affected Agent
path. The maintainer renewed this Query-only acceptance through 2026-11-20 UTC.
The gate refuses from 2026-11-21 UTC, and all other dependency checks remain active.

On 2026-11-07, the Platform 0.24.0 adoption on the 0.93.4 release source
was reviewed with Cargo.lock SHA-256 `08b5e8968cc18637b8a84df78c142506101fc2e533c73d47be4179890e81ca7b`.
Locked, offline native graph inspection still reaches Agent 0.49.3 →
elliptic-curve 0.13.8 → crypto-bigint 0.5.5. The two Platform facade versions
resolve to the same Agent and canonical Principal supplier identities. Removing
Platform’s separate offline derivation capability does not remove Agent’s
affected crypto path. This adoption neither fixes the advisory nor renews its
acceptance; refusal remains scheduled for 2026-11-21 UTC.

On 2026-11-07, the subsequent Build 0.78.0 and Host Foundation 0.75.0
adoption was reviewed with Cargo.lock SHA-256
`36bf98741480210cac49b656ef8c61469bff0195c3693db1247868421b6ea38a`. The locked, offline
native graph retains Host Tools 0.83.0, Agent 0.49.3, elliptic-curve 0.13.8
and crypto-bigint 0.5.5. The bounded Host HTTP, filesystem and build updates
do not replace that Agent crypto path or extend the 2026-11-21 review deadline.

On 2026-11-07, the Host Tools 0.84.0 release candidate was reviewed with
Cargo.lock SHA-256 `fc42e486a0c62338abd814018bee2028cd35b532a2f91afdf81a92c27b538ed5`.
The locked native graph keeps Agent 0.49.3 → elliptic-curve 0.13.8 →
crypto-bigint 0.5.5. Host Tools now shares Foundation 0.75 with Query;
Testkit retains its separate Foundation 0.74 dependencies. This supplier
alignment does not correct the advisory or extend the existing acceptance.

On 2026-11-07, Host Tools 0.85.0 and Testkit 0.92.0 were reviewed with
Cargo.lock SHA-256 `2a30f34115f67d2adb534f81a88d9cc6703d160cfdae140186d2c67e77d290ce`.
The native graph now contains only Platform 0.24.0 and Foundation 0.75;
Agent 0.49.3 still reaches elliptic-curve 0.13.8 and crypto-bigint 0.5.5.
Removing duplicate supplier versions does not fix the advisory or extend its
existing 2026-11-21 review deadline.

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
