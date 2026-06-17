//! Module: sns::report::source::traits::params
//!
//! Responsibility: SNS governance-parameters source contract.
//! Does not own: live governance transport, params assembly, or rendering.
//! Boundary: extends deployed SNS lookup sources with parameter fetching.

use super::super::{MainnetSns, SnsFetchRequest};
use super::list::SnsListSource;
use crate::sns::report::{SnsGovernanceParameters, SnsHostError};

///
/// SnsParamsSource
///
/// Source contract for fetching governance parameters for one deployed SNS.
///

pub(in crate::sns::report) trait SnsParamsSource: SnsListSource {
    /// Fetch SNS governance parameters for one resolved SNS.
    fn fetch_sns_params(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError>;
}
