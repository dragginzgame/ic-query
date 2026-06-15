use super::super::common::{command_icp_root, command_unix_secs};
use crate::{
    cli::{common::write_text_or_json, help::print_help_or_version},
    sns::{
        commands::{
            SnsCommandError, options::SnsNeuronsRefreshOptions, spec::sns_neurons_refresh_usage,
        },
        report::{
            SnsNeuronsRefreshRequest, refresh_sns_neurons_cache, sns_neurons_refresh_report_text,
        },
    },
    version_text,
};
use std::ffi::OsString;

pub(super) fn run_sns_neurons_refresh<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_neurons_refresh_usage, version_text()) {
        return Ok(());
    }
    let options = SnsNeuronsRefreshOptions::parse(args)?;
    let format = options.lookup.format;
    let request = SnsNeuronsRefreshRequest {
        network: options.lookup.network,
        source_endpoint: options.lookup.source_endpoint,
        now_unix_secs: command_unix_secs()?,
        input: options.lookup.input,
        icp_root: command_icp_root()?,
        page_size: options.page_size,
        max_pages: options.max_pages,
    };
    let report = refresh_sns_neurons_cache(&request)?;
    write_text_or_json(format, &report, sns_neurons_refresh_report_text)
}
