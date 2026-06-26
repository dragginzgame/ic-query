#[cfg(feature = "host")]
mod error;
mod report;
mod request;

#[cfg(feature = "host")]
pub use error::NnsDataCenterHostError;
#[cfg(feature = "host")]
pub use report::NnsDataCenterRefreshReport;
pub use report::{NnsDataCenterInfoReport, NnsDataCenterListReport, NnsDataCenterRow};
#[cfg(feature = "host")]
pub use request::NnsDataCenterRefreshRequest;
pub use request::{NnsDataCenterCacheRequest, NnsDataCenterInfoRequest, NnsDataCenterListRequest};
