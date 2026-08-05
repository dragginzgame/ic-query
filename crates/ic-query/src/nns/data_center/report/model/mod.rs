#[cfg(feature = "nns-host")]
mod error;
mod report;

#[cfg(feature = "nns-host")]
pub use error::NnsDataCenterHostError;
#[cfg(feature = "nns-host")]
pub use report::NnsDataCenterRefreshReport;
pub use report::{NnsDataCenterInfoReport, NnsDataCenterListReport, NnsDataCenterRow};
