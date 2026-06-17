//! Module: sns::report::live::client::params
//!
//! Responsibility: live SNS params source implementation.
//! Does not own: governance query construction, report assembly, or rendering.
//! Boundary: delegates the params source trait to live fetch helpers.

use super::LiveSnsSource;
use crate::sns::report::{
    MainnetSns, SnsFetchRequest, SnsGovernanceParameters, SnsHostError, SnsParamsSource,
    live::fetch::fetch_mainnet_sns_params,
};

impl SnsParamsSource for LiveSnsSource {
    fn fetch_sns_params(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError> {
        fetch_mainnet_sns_params(request, sns)
    }
}
