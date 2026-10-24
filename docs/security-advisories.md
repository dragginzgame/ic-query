# Local advisory acceptance

## RUSTSEC-2026-0387

Accepted on 2026-10-24 at the maintainer's direction to retain upstream IC
versions. Review by **2026-11-07**, and on the next Platform/Agent adoption.
This is a temporary risk acceptance, not a vulnerability fix or a finding that
all library consumers are unaffected. The dependency gate refuses execution from
2026-11-07 UTC until the acceptance is explicitly reviewed or removed.

The selected path is Platform 0.6.13 → Agent 0.49.3 → elliptic-curve 0.13.8 →
crypto-bigint 0.5.5, including Agent's k256/p256 paths. elliptic-curve requires
crypto-bigint 0.5; the patched 0.7.6 cannot replace it through a compatible lock
update. Do not fork cryptography or force an incompatible override here.

Query's built-in clients use `src/agent.rs` in `crates/ic-query` and Host's
`agent_builder`/`EndpointAgent`, retaining Agent's default anonymous identity.
Agent 0.49.3's `AnonymousIdentity::sign` produces no signature or public key.
Query does not load signing keys. However, library interfaces accept configured
Agents and dispatch queries through them; caller-supplied ECDSA identities can
exercise secret arithmetic. Their exposure is not excluded by this review.

The [advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html) concerns
secret-dependent timing, with exploitability dependent on operations, compiler,
CPU and observation. This source review is not a side-channel qualification.

The local audit ignores only this advisory in addition to its existing exceptions;
other advisories and warnings still fail the gate. Keep
[Platform #50](https://github.com/dragginzgame/ic-platform/issues/50) open. Remove
this exception when a compatible upstream correction eliminates affected versions
from Query's resolved graph, then rerun the current-feed dependency gate and
relevant Agent/MSRV checks. Do not transfer this acceptance to other consumers.
