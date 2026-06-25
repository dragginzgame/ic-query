//! Module: subnet_catalog::model::classification
//!
//! Exposes stable subnet classification enums for catalog records and reports.

mod geographic;
mod kind;
mod source;
mod specialization;

pub use geographic::GeographicScope;
pub use kind::SubnetKind;
pub use source::ClassificationSource;
pub use specialization::SubnetSpecialization;
