//! Module: sns::commands::spec::values::list
//!
//! Responsibility: clap value enum for SNS list sorting.
//! Does not own: SNS list report sorting or command runtime behavior.
//! Boundary: converts CLI sort values into report-model sort values.

use crate::sns::report::SnsListSort;
use clap::ValueEnum;

///
/// SnsListSortArg
///
/// Command-local clap value accepted by `icq sns list --sort`.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(in crate::sns::commands) enum SnsListSortArg {
    Id,
    Name,
}

impl From<SnsListSortArg> for SnsListSort {
    fn from(value: SnsListSortArg) -> Self {
        match value {
            SnsListSortArg::Id => Self::Id,
            SnsListSortArg::Name => Self::Name,
        }
    }
}
