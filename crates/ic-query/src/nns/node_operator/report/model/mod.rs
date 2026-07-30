#[cfg(feature = "host")]
mod error;
mod report;

#[cfg(feature = "host")]
pub use error::NnsNodeOperatorHostError;
#[cfg(feature = "host")]
pub use report::NnsNodeOperatorRefreshReport;
pub use report::{NnsNodeOperatorInfoReport, NnsNodeOperatorListReport, NnsNodeOperatorRow};
