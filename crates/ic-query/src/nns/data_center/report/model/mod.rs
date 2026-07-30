#[cfg(feature = "host")]
mod error;
mod report;

#[cfg(feature = "host")]
pub use error::NnsDataCenterHostError;
#[cfg(feature = "host")]
pub use report::NnsDataCenterRefreshReport;
pub use report::{NnsDataCenterInfoReport, NnsDataCenterListReport, NnsDataCenterRow};
