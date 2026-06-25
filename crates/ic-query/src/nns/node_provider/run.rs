use super::{reports::NnsNodeProviderReports, spec::NODE_PROVIDER_SPEC};
use crate::nns::{NnsCommandError, leaf, node_provider::report::DEFAULT_NNS_SOURCE_ENDPOINT};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    leaf::run_cached_leaf(
        args,
        &NODE_PROVIDER_SPEC,
        DEFAULT_NNS_SOURCE_ENDPOINT,
        NnsNodeProviderReports,
    )
}
