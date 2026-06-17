//! Module: sns::report::source::traits::token
//!
//! Responsibility: SNS token metadata source contract.
//! Does not own: live ledger transport, token report assembly, or rendering.
//! Boundary: extends deployed SNS lookup sources with token metadata fetching.

use super::super::{MainnetSns, MainnetSnsToken, SnsFetchRequest};
use super::list::SnsListSource;
use crate::sns::report::SnsHostError;

///
/// SnsTokenSource
///
/// Source contract for fetching token metadata for one deployed SNS.
///

pub(in crate::sns::report) trait SnsTokenSource: SnsListSource {
    /// Fetch SNS ledger token metadata for one resolved SNS.
    fn fetch_sns_token(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsToken, SnsHostError>;
}
