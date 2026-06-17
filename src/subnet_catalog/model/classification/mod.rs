//! Module: subnet_catalog::model::classification
//!
//! Responsibility: expose subnet classification enums used by catalog records and reports.
//!
//! Does not own: classification derivation, registry fetching, or CLI parsing.
//!
//! Boundary: keeps the stable enum surface for subnet kind, specialization,
//! geography, and metadata provenance.

mod geographic;
mod kind;
mod source;
mod specialization;

pub use geographic::GeographicScope;
pub use kind::SubnetKind;
pub use source::ClassificationSource;
pub use specialization::SubnetSpecialization;
