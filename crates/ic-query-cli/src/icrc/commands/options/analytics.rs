//! Clap schemas and request projection for bounded Dashboard analytics.
use super::super::analytics::AnalyticsAccountInfoOptions;
use super::{IcrcLedgerOptions, parsed_options};
use crate::cli::common::{OutputFormat, resolve_time_bounds};
use clap::{
    ArgMatches, Args,
    builder::{PossibleValuesParser, RangedU64ValueParser, TypedValueParser},
};
use ic_query::ic::{
    DEFAULT_ICRC_TOKEN_VALUE_WINDOW_SECS, DEFAULT_ICRC_TOTAL_SUPPLY_STEP_SECS,
    DEFAULT_ICRC_TOTAL_SUPPLY_WINDOW_SECS, IcIcrcAccountInfoRequest, IcIcrcAccountListQuery,
    IcIcrcAccountListRequest, IcIcrcAccountSort, IcIcrcHolderListQuery, IcIcrcHolderListRequest,
    IcIcrcHolderSort, IcIcrcTokenValueQuery, IcIcrcTokenValueRequest, IcIcrcTotalSupplyQuery,
    IcIcrcTotalSupplyRequest, MAX_ICRC_INDEX_PAGE_ROWS, MAX_ICRC_TOKEN_VALUE_ROWS,
    MIN_ICRC_ANALYTICS_TIMESTAMP,
};

///
/// IndexPageOptions
///
/// Shared opaque cursors and bounded row count for one index page.
///

#[derive(Args)]
#[group(skip)]
struct IndexPageOptions {
    #[arg(
        long,
        value_name = "cursor",
        conflicts_with = "before",
        help = "Opaque exclusive forward cursor returned by an earlier page"
    )]
    after: Option<String>,
    #[arg(
        long,
        value_name = "cursor",
        help = "Opaque exclusive backward cursor returned by an earlier page"
    )]
    before: Option<String>,
    #[arg(long, value_name = "rows", default_value = "20", value_parser = RangedU64ValueParser::<u16>::new().range(1..=u64::from(MAX_ICRC_INDEX_PAGE_ROWS)), help = "Maximum rows requested from this one page")]
    limit: u16,
}

///
/// AccountListOptions
///
/// Clap-owned account index page, ordering and owner selection.
///

#[derive(Args)]
#[group(skip)]
pub(in crate::icrc) struct AccountListOptions {
    #[command(flatten)]
    page: IndexPageOptions,
    #[arg(long, value_name = "field", default_value = "id", allow_hyphen_values = true, value_parser = crate::cli::clap::choice_parser(IcIcrcAccountSort::ALL, IcIcrcAccountSort::as_api_value), help = "Stable upstream page sort; a leading '-' selects descending order")]
    sort_by: IcIcrcAccountSort,
    #[arg(long, value_name = "principal", value_parser = super::principal_text_value_parser(), help = "Restrict accounts to one canonical owner principal")]
    owner: Option<String>,
}

///
/// HolderListOptions
///
/// Clap-owned principal-level holder page and ordering.
///

#[derive(Args)]
#[group(skip)]
pub(in crate::icrc) struct HolderListOptions {
    #[command(flatten)]
    page: IndexPageOptions,
    #[arg(long, value_name = "field", default_value = "principal", allow_hyphen_values = true, value_parser = crate::cli::clap::choice_parser(IcIcrcHolderSort::ALL, IcIcrcHolderSort::as_api_value), help = "Stable upstream page sort; a leading '-' selects descending order")]
    sort_by: IcIcrcHolderSort,
}

///
/// TokenValueOptions
///
/// Clap-owned token-value bounds; omitted times resolve against the caller clock.
///

#[derive(Args)]
#[group(skip)]
pub(in crate::icrc) struct TokenValueOptions {
    #[arg(
        long,
        value_name = "unix-seconds",
        help = "Series start; defaults to 24 hours before --end"
    )]
    start: Option<u64>,
    #[arg(
        long,
        value_name = "unix-seconds",
        help = "Series end; defaults to the current time"
    )]
    end: Option<u64>,
    #[arg(long, value_name = "rows", default_value = "1000", value_parser = RangedU64ValueParser::<u16>::new().range(1..=u64::from(MAX_ICRC_TOKEN_VALUE_ROWS)), help = "Maximum rows requested; reaching it reports possible truncation")]
    limit: u16,
}

///
/// TotalSupplyOptions
///
/// Clap-owned total-supply bounds and upstream-supported sampling intervals.
///

#[derive(Args)]
#[group(skip)]
pub(in crate::icrc) struct TotalSupplyOptions {
    #[arg(long, value_name = "unix-seconds", value_parser = RangedU64ValueParser::<u64>::new().range(MIN_ICRC_ANALYTICS_TIMESTAMP..), help = "Inclusive series start; defaults to 30 days before --end")]
    start: Option<u64>,
    #[arg(long, value_name = "unix-seconds", value_parser = RangedU64ValueParser::<u64>::new().range(MIN_ICRC_ANALYTICS_TIMESTAMP..), help = "Inclusive series end; defaults to the current time")]
    end: Option<u64>,
    #[arg(long, value_name = "seconds", default_value_t = DEFAULT_ICRC_TOTAL_SUPPLY_STEP_SECS, hide_default_value = true, value_parser = PossibleValuesParser::new(["3600", "86400"]).try_map(|value| value.parse::<u32>()), help = "Observation interval accepted by the official API; defaults to 86400")]
    step: u32,
}

pub(in crate::icrc) fn analytics_account_list_request(
    matches: &ArgMatches,
    now: u64,
) -> (OutputFormat, IcIcrcAccountListRequest) {
    let options: AccountListOptions = parsed_options(matches);
    let (format, dashboard, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_dashboard_request(now);
    let mut query = IcIcrcAccountListQuery::new(options.page.limit, options.sort_by);
    query.owner = options.owner;
    query.after = options.page.after;
    query.before = options.page.before;
    (
        format,
        IcIcrcAccountListRequest::new(dashboard, ledger, query),
    )
}

pub(in crate::icrc) fn analytics_account_info_request(
    matches: &ArgMatches,
    now: u64,
) -> (OutputFormat, IcIcrcAccountInfoRequest) {
    let options: AnalyticsAccountInfoOptions = parsed_options(matches);
    let (format, dashboard, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_dashboard_request(now);
    (
        format,
        IcIcrcAccountInfoRequest::new(dashboard, ledger, options.account_id),
    )
}

pub(in crate::icrc) fn analytics_holder_list_request(
    matches: &ArgMatches,
    now: u64,
) -> (OutputFormat, IcIcrcHolderListRequest) {
    let options: HolderListOptions = parsed_options(matches);
    let (format, dashboard, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_dashboard_request(now);
    let mut query = IcIcrcHolderListQuery::new(options.page.limit, options.sort_by);
    query.after = options.page.after;
    query.before = options.page.before;
    (
        format,
        IcIcrcHolderListRequest::new(dashboard, ledger, query),
    )
}

pub(in crate::icrc) fn analytics_token_value_request(
    matches: &ArgMatches,
    now: u64,
) -> (OutputFormat, IcIcrcTokenValueRequest) {
    let options: TokenValueOptions = parsed_options(matches);
    let (format, dashboard, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_dashboard_request(now);
    let (start, end) = resolve_time_bounds(
        options.start,
        options.end,
        now,
        DEFAULT_ICRC_TOKEN_VALUE_WINDOW_SECS,
        0,
    );
    (
        format,
        IcIcrcTokenValueRequest::new(
            dashboard,
            ledger,
            IcIcrcTokenValueQuery::new(start, end, options.limit),
        ),
    )
}

pub(in crate::icrc) fn analytics_total_supply_request(
    matches: &ArgMatches,
    now: u64,
) -> (OutputFormat, IcIcrcTotalSupplyRequest) {
    let options: TotalSupplyOptions = parsed_options(matches);
    let (format, dashboard, ledger) =
        IcrcLedgerOptions::from_matches(matches).into_dashboard_request(now);
    let (start, end) = resolve_time_bounds(
        options.start,
        options.end,
        now,
        DEFAULT_ICRC_TOTAL_SUPPLY_WINDOW_SECS,
        MIN_ICRC_ANALYTICS_TIMESTAMP,
    );
    (
        format,
        IcIcrcTotalSupplyRequest::new(
            dashboard,
            ledger,
            IcIcrcTotalSupplyQuery::new(start, end, options.step),
        ),
    )
}
