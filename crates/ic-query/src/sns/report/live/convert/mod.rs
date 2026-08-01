//! Module: sns::report::live::convert
//!
//! Responsibility: group live SNS wire-to-domain conversion helpers.
//! Does not own: live transport, Candid wire type definitions, cache IO, or rendering.
//! Boundary: re-exports converters used by live fetch and report builders.

mod canisters;
mod common;
mod metadata;
mod neurons;
mod proposals;
mod sns;
mod swap;
mod upgrade;

pub(super) use canisters::mainnet_sns_canister_inventory;
pub(super) use metadata::metadata_error_summary;
pub(super) use neurons::sns_neuron_row;
pub(super) use proposals::sns_proposal_row;
pub(super) use sns::{mainnet_sns_canisters_from_deployed_sns, mainnet_sns_metadata_from_response};
pub(super) use swap::{sns_swap_derived_state, sns_swap_lifecycle, sns_swap_sale_parameters};
pub(super) use upgrade::{sns_pending_upgrade, sns_version};
