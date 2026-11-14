//! Read-only IC infrastructure evidence for Canic application topology decisions.
//!
//! [`placement`] supplies a bounded mainnet canister-to-subnet observation;
//! [`subnet`] observes node identities in subnet-certified state.
//! Further topology, capability, resource, cost and change reports remain planned,
//! including scoped uptime and subnet-maintenance evidence. Native use requires
//! a Tokio runtime with time and IO enabled.
//!
//! SNS migration and NNS governance are outside this package's scope. Canic owns
//! planning and execution; this library will supply evidence and comparisons.

mod mainnet;
pub use mainnet::ReadError;
pub mod placement;
pub mod subnet;
