use crate::cli::common::SourceOutputOptions;
use ic_query::subnet_catalog::{GeographicScope, SubnetKind, SubnetSpecialization};

pub(super) const DEFAULT_RANGE_LIMIT_ARG: &str = "50";
pub(super) const INFO_INPUT_VALUE_NAME: &str = "subnet|canister|subnet-prefix";
pub(super) const INFO_INPUT_HELP: &str = "Subnet/canister principal or unique subnet prefix";

///
/// SubnetListOptions
///
/// Typed catalog filters and routing-range display controls.
///

#[derive(clap::Args)]
#[group(skip)]
pub(in crate::nns) struct SubnetListOptions {
    #[arg(
        long,
        value_name = "kind",
        help = "Filter by subnet kind: application, cloud_engine, system, or unknown"
    )]
    pub(in crate::nns) kind: Option<SubnetKind>,
    #[arg(
        long,
        value_name = "specialization",
        help = "Filter by specialization: none, fiduciary, european, or unknown"
    )]
    pub(in crate::nns) specialization: Option<SubnetSpecialization>,
    #[arg(
        long,
        value_name = "scope",
        help = "Filter by geographic scope: global, europe, or unknown"
    )]
    pub(in crate::nns) geo: Option<GeographicScope>,
    #[command(flatten)]
    pub(in crate::nns) source: SourceOutputOptions,
    #[arg(
        id = "show-ranges",
        long,
        help = "Show cached routing ranges after the subnet table"
    )]
    pub(in crate::nns) show_ranges: bool,
    #[arg(
        long,
        help = "Show full subnet principals and catalog metadata in text output"
    )]
    pub(in crate::nns) verbose: bool,
    #[arg(id = "range-limit", long, value_name = "n", default_value = DEFAULT_RANGE_LIMIT_ARG, value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1u64..), help = "Maximum routing ranges to show per subnet in text output")]
    pub(in crate::nns) range_limit: usize,
    #[arg(
        id = "range-offset",
        long,
        value_name = "n",
        default_value = "0",
        help = "Routing range offset for text output"
    )]
    pub(in crate::nns) range_offset: usize,
}
