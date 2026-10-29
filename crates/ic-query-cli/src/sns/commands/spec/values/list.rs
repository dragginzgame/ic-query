//! Module: sns::commands::spec::values::list
//!
//! Responsibility: clap value enum for SNS list sorting.
//! Does not own: SNS list report sorting or command runtime behavior.
//! Boundary: converts CLI sort values into report-model sort values.

use crate::cli::value_enum_with_conversion;
use ic_query::sns::SnsListSort;

value_enum_with_conversion! {
    ///
    /// SnsListSortArg
    ///
    /// Command-local clap value accepted by `icq sns list --sort`.
    ///

    pub(in crate::sns::commands) enum SnsListSortArg => SnsListSort {
        #[default]
        Id,
        Name,
    }
}
