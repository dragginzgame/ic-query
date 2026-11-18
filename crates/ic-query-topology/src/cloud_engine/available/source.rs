//! Only the selected public status and new-engine page requests.

use super::{AvailableError, Observation, PAGE_METHOD, PAGE_SIZE, STATUS_METHOD};
use ic_platform::native::ReadClient;
use ic_platform_protocol::{cloud_engine as wire, cloud_engine::encode_empty_args};
use ic_platform_types::Principal;

pub(super) trait Source {
    async fn status(&self) -> Result<Observation, AvailableError>;
    async fn page(&self, start: Option<Principal>) -> Result<Observation, AvailableError>;
}

pub(super) struct MainnetSource(pub ReadClient);

pub(super) fn page_argument(start: Option<Principal>) -> Result<Vec<u8>, wire::EncodeError> {
    wire::encode_list_available_nodes_page_args(
        &wire::ListAvailableNodesPageArgs {
            rejoining: None,
            start,
            limit: Some((PAGE_SIZE as u64).into()),
        },
        512,
    )
}

impl Source for MainnetSource {
    async fn status(&self) -> Result<Observation, AvailableError> {
        Ok(super::super::query(&self.0, STATUS_METHOD, encode_empty_args(64)?).await?)
    }

    async fn page(&self, start: Option<Principal>) -> Result<Observation, AvailableError> {
        Ok(super::super::query(&self.0, PAGE_METHOD, page_argument(start)?).await?)
    }
}
