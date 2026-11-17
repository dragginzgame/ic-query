//! Fixed public query source; no arbitrary-call public API.

pub(super) use super::super::Observation;
use super::PriceError;
use ic_platform::native::ReadClient;
use ic_platform_protocol::cloud_engine::encode_empty_args;

pub(super) trait Source {
    async fn query(&self, method: &'static str) -> Result<Observation, PriceError>;
}

pub(super) struct MainnetSource(pub ReadClient);

impl Source for MainnetSource {
    async fn query(&self, method: &'static str) -> Result<Observation, PriceError> {
        let argument = encode_empty_args(64)?;
        Ok(super::super::query(&self.0, method, argument).await?)
    }
}
