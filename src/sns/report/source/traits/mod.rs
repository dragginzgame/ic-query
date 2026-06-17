//! Module: sns::report::source::traits
//!
//! Responsibility: group SNS report source traits.
//! Does not own: live transport, cache IO, report assembly, or rendering.
//! Boundary: re-exports source contracts used by report builders and tests.

mod list;
mod neurons;
mod params;
mod proposals;
mod token;

pub(in crate::sns::report) use list::SnsListSource;
pub(in crate::sns::report) use neurons::SnsNeuronsSource;
pub(in crate::sns::report) use params::SnsParamsSource;
pub(in crate::sns::report) use proposals::{SnsProposalSource, SnsProposalsSource};
pub(in crate::sns::report) use token::SnsTokenSource;
