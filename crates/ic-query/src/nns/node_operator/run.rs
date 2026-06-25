use super::{reports::NnsNodeOperatorReports, spec::NODE_OPERATOR_SPEC};
use crate::nns::{
    NnsCommandError, leaf, node_operator::report::DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    leaf::run_cached_leaf(
        args,
        &NODE_OPERATOR_SPEC,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        NnsNodeOperatorReports,
    )
}
