//! Read-only IC infrastructure evidence for Canic application topology decisions.
//!
//! [`placement`] supplies a bounded mainnet canister-to-subnet observation;
//! [`subnet`] observes node identities in subnet-certified state, and
//! [`subnet::list`] enumerates the Registry's registered subnet identities.
//! [`subnet::metrics`] supplies certified raw resource values for one subnet.
//! [`subnet::configuration`] reads registered membership, version and halt flags;
//! configuration is distinct from observed running state or availability.
//! [`subnet::membership`] compares registered members and observed node keys
//! while retaining their separate evidence and non-atomic acquisition.
//! [`node`] reads registered operator, chip and HostOS data without attestation.
//! Further topology, capability, resource, cost and change reports remain planned,
//! including scoped uptime and subnet-maintenance evidence. Native use requires
//! a Tokio runtime with time and IO enabled.
//!
//! SNS migration and NNS governance are outside this package's scope. Canic owns
//! planning and execution; this library will supply evidence and comparisons.

mod mainnet;
pub use mainnet::ReadError;
pub mod node;
pub mod placement;
pub mod subnet;
