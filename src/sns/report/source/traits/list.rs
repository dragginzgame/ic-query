//! Module: sns::report::source::traits::list
//!
//! Responsibility: deployed SNS list source contract.
//! Does not own: live SNS-W transport, lookup sorting, or report assembly.
//! Boundary: lets report builders fetch deployed SNS inventory from fixtures or live sources.

use crate::sns::report::{MainnetSnsList, SnsFetchRequest, SnsHostError};

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
