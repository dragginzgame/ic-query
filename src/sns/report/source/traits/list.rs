//! Module: sns::report::source::traits::list
//!
//! Responsibility: deployed SNS list source contract.
//! Does not own: live SNS-W transport, lookup sorting, or report assembly.
//! Boundary: lets report builders fetch deployed SNS inventory from fixtures or live sources.

use super::super::{MainnetSnsList, SnsFetchRequest};
use crate::sns::report::SnsHostError;

///
/// SnsListSource
///
/// Source contract for fetching deployed SNS inventory.
///

pub(in crate::sns::report) trait SnsListSource {
    /// Fetch deployed SNS instances for one source endpoint and network.
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError>;
}
