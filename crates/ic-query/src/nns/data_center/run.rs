use super::{reports::NnsDataCenterReports, spec::DATA_CENTER_SPEC};
use crate::nns::{
    NnsCommandError, data_center::report::DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT, leaf,
};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    leaf::run_cached_leaf(
        args,
        &DATA_CENTER_SPEC,
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
        NnsDataCenterReports,
    )
}
