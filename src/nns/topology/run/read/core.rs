use crate::{
    cli::help::print_help_or_version,
    nns::{
        NnsCommandError, command_icp_root, now_unix_secs, topology::options::TopologyReadOptions,
        write_text_or_json,
    },
    version_text,
};
use serde::Serialize;
use std::ffi::OsString;

pub(super) trait TopologyReadRunner {
    type Options: TopologyReadOptions<Self::Request>;
    type Request;
    type Report: Serialize;
    type HostError: Into<NnsCommandError>;

    fn usage() -> String;
    fn build_report(request: &Self::Request) -> Result<Self::Report, Self::HostError>;
    fn render_text(report: &Self::Report) -> String;
}

pub(super) fn run_topology_read<I, Runner>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
    Runner: TopologyReadRunner,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, Runner::usage, version_text()) {
        return Ok(());
    }
    let options = Runner::Options::parse_args(args)?;
    let format = options.format();
    let icp_root = command_icp_root()?;
    let request = options.into_request(icp_root, now_unix_secs()?);
    let report = Runner::build_report(&request).map_err(Into::into)?;
    write_text_or_json(format, &report, Runner::render_text)
}
