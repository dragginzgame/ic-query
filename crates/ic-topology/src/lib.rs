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
//! [`node::changes`] compares those raw fields without inferring TEE readiness.
//! [`subnet::nodes`] collects those records with explicit per-node outcomes.
//! [`subnet::distribution`] joins providers and registered data centers at one
//! requested Registry version, retaining independent relationship coverage.
//! [`subnet::changes`] compares registered configurations without inferring events.
//! [`cloud_engine::prices`] reads daily marketplace prices and separate fee evidence.
//! [`cloud_engine::available`] traverses public candidate nodes with page evidence
//! and explicit stop reasons, without claiming an atomic inventory or capacity.
//! [`cloud_engine::candidates`] matches those nodes to public declaration scopes
//! without selecting an effective tariff or deployment quote.
//! [`cloud_engine::engine`] reads one selected engine's subnet/operator and
//! separates engine-record node identities from embedded Registry-copy membership.
//! Further topology, capability, resource, cost and change reports remain planned,
//! including scoped uptime and subnet-maintenance evidence. Native use requires
//! a Tokio runtime with time and IO enabled.
//!
//! SNS migration and NNS governance are outside this package's scope. Canic owns
//! planning and execution; this library will supply evidence and comparisons.

pub mod cloud_engine;
mod mainnet;
pub use mainnet::ReadError;
pub mod node;
pub mod placement;
pub mod subnet;
