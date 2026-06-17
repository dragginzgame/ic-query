//! Module: sns::report::live::client::token
//!
//! Responsibility: live SNS token source implementation.
//! Does not own: token query construction, report assembly, or rendering.
//! Boundary: delegates the token source trait to live fetch helpers.

use super::LiveSnsSource;
use crate::sns::report::{
    MainnetSns, MainnetSnsToken, SnsFetchRequest, SnsHostError, SnsTokenSource,
    live::fetch::fetch_mainnet_sns_token,
};

impl SnsTokenSource for LiveSnsSource {
    fn fetch_sns_token(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsToken, SnsHostError> {
        fetch_mainnet_sns_token(request, sns)
    }
}
