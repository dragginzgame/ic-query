//! Module: sns::report::live::client::list
//!
//! Responsibility: live deployed SNS list source implementation.
//! Does not own: SNS-W query construction, report assembly, or rendering.
//! Boundary: delegates the list source trait to live fetch helpers.

use super::LiveSnsSource;
use crate::sns::report::{
    MainnetSnsList, SnsFetchRequest, SnsHostError, SnsListSource,
    live::fetch::fetch_mainnet_sns_list,
};

impl SnsListSource for LiveSnsSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        fetch_mainnet_sns_list(request)
    }
}
