use super::*;

macro_rules! delegate_sns_discovery {
    ($source:ty) => {
        impl SnsDiscoverySource for $source {
            fn fetch_sns_inventory(
                &self,
                request: &SnsSourceRequest,
            ) -> Result<MainnetSnsInventory, SnsHostError> {
                FixtureSnsDiscoverySource.fetch_sns_inventory(request)
            }

            fn fetch_sns_metadata(
                &self,
                request: &SnsSourceRequest,
                targets: &[MainnetSnsCanisters],
            ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
                FixtureSnsDiscoverySource.fetch_sns_metadata(request, targets)
            }
        }
    };
}

mod canisters;
mod fixtures;
mod list;
mod metrics;
mod neurons;
mod params;
mod proposals;
mod reward;
mod swap;
mod token;
mod upgrade;
