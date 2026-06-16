use super::super::{
    commands::topology_refresh_usage,
    options::TopologyRefreshOptions,
    report::{
        NnsTopologyRefreshRequest, nns_topology_refresh_report_text, refresh_nns_topology_report,
    },
};
use crate::{
    cli::help::print_help_or_version,
    nns::{NnsCommandError, command_icp_root, now_unix_secs, write_text_or_json},
    version_text,
};
use std::ffi::OsString;

pub(super) fn run_topology_refresh<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, topology_refresh_usage, version_text()) {
        return Ok(());
    }
    let options = TopologyRefreshOptions::parse(args)?;
    let format = options.format;
    let icp_root = command_icp_root()?;
    let request = NnsTopologyRefreshRequest {
        icp_root,
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
        lock_stale_after_seconds: options.lock_stale_after_seconds,
        dry_run: options.dry_run,
    };
    let report = refresh_nns_topology_report(&request)?;
    write_text_or_json(format, &report, nns_topology_refresh_report_text)
}
