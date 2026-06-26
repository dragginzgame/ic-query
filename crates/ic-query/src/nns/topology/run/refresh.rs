use crate::nns::{
    NnsCommandError, command_args, command_icp_root, now_unix_secs,
    topology::{
        commands::topology_refresh_usage,
        options::TopologyRefreshOptions,
        report::{
            NnsTopologyRefreshRequest, nns_topology_refresh_report_text,
            refresh_nns_topology_report,
        },
    },
    write_text_or_json,
};
use std::ffi::OsString;

pub(super) fn run_topology_refresh<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, topology_refresh_usage) else {
        return Ok(());
    };
    let options = TopologyRefreshOptions::parse(args)?;
    let format = options.format;
    let icp_root = command_icp_root()?;
    let request = NnsTopologyRefreshRequest::new(
        icp_root,
        options.network,
        options.source_endpoint,
        now_unix_secs()?,
        options.lock_stale_after_seconds,
    )
    .with_dry_run(options.dry_run);
    let report = refresh_nns_topology_report(&request)?;
    write_text_or_json(format, &report, nns_topology_refresh_report_text)
}
