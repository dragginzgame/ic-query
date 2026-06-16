use crate::nns::{
    NnsCommandError, command_args, command_icp_root, now_unix_secs,
    topology::{options::TopologyReadOptions, report::NnsTopologyHostError},
    write_text_or_json,
};
use serde::Serialize;
use std::ffi::OsString;

pub(super) fn run_topology_read<Options, Request, Report>(
    args: Vec<OsString>,
    usage: fn() -> String,
    build_report: fn(&Request) -> Result<Report, NnsTopologyHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), NnsCommandError>
where
    Options: TopologyReadOptions<Request>,
    Report: Serialize,
{
    let Some(args) = command_args(args, usage) else {
        return Ok(());
    };
    let options = Options::parse_args(args)?;
    let format = options.format();
    let icp_root = command_icp_root()?;
    let request = options.into_request(icp_root, now_unix_secs()?);
    let report = build_report(&request)?;
    write_text_or_json(format, &report, render_text)
}
