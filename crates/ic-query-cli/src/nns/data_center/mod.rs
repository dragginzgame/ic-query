//! Module: nns::data_center
//!
//! Responsibility: assemble data-center CLI specification, reports, and dispatch.
//! Does not own: reusable report construction or cache mechanics.
//! Boundary: adapts data-center arguments to the typed library API.

mod reports;
mod run;
mod spec;
#[cfg(test)]
pub(in crate::nns) mod test_helpers;

pub(super) use run::run;
