//! Module: sns::report::live::convert
//!
//! Responsibility: group live SNS wire-to-domain conversion helpers.
//! Does not own: live transport, Candid wire type definitions, cache IO, or rendering.
//! Boundary: re-exports converters used by live fetch and report builders.

mod common;
mod metadata;
mod neurons;
mod proposals;
mod sns;

pub(in crate::sns::report) use metadata::metadata_row;
pub(super) use metadata::{index_principal_error_text, metadata_error_summary};
pub(super) use neurons::{sns_neuron_cursor, sns_neuron_row};
pub(super) use proposals::sns_proposal_row;
pub(super) use sns::{
    mainnet_sns_canisters_from_deployed_sns, mainnet_sns_from_canisters_and_metadata,
};
