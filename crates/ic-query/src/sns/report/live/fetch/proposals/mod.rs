//! Module: sns::report::live::fetch::proposals
//!
//! Responsibility: fetch SNS governance proposals.
//! Does not own: lookup resolution, cache storage, report assembly, or rendering.
//! Boundary: queries direct, bounded, and paged proposal data from governance.

mod list;
mod single;

pub(in crate::sns::report::live) use list::{
    fetch_mainnet_sns_proposal_page, fetch_mainnet_sns_proposals,
};
pub(in crate::sns::report::live) use single::fetch_mainnet_sns_proposal;
