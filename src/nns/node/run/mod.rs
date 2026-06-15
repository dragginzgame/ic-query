mod cache;
mod info;
mod list;
mod refresh;

use super::commands::NODE_SPEC;
use crate::nns::{NnsCommandError, leaf};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    leaf::run_leaf(
        args,
        &NODE_SPEC,
        list::run_node_list,
        info::run_node_info,
        refresh::run_node_refresh,
    )
}
