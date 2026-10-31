use super::*;

macro_rules! delegate_sns_discovery {
    ($source:ty) => {
        impl SnsDiscoverySource for $source {
            fn fetch_sns_inventory<'a>(
                &'a self,
                request: &'a $crate::report::CollectionSourceRequest,
            ) -> crate::report::SourceFuture<'a, Result<MainnetSnsInventory, SnsHostError>> {
                FixtureSnsDiscoverySource.fetch_sns_inventory(request)
            }

            fn fetch_sns_metadata<'a>(
                &'a self,
                request: &'a $crate::report::CollectionSourceRequest,
                targets: &'a [MainnetSnsCanisters],
            ) -> crate::report::SourceFuture<'a, Result<Vec<MainnetSnsMetadata>, SnsHostError>>
            {
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
