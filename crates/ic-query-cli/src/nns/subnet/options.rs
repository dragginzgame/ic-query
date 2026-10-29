use crate::{
    cli::clap::{required_string, required_typed, typed_option},
    nns::leaf::NnsCommonOptions,
};
use clap::ArgMatches;
use ic_query::subnet_catalog::{ResolveAs, SubnetCatalogFilters};

///
/// CatalogListOptions
///
/// Parsed options accepted by `icq nns subnet list`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct CatalogListOptions {
    pub(in crate::nns) common: NnsCommonOptions,
    pub(in crate::nns) filters: SubnetCatalogFilters,
    pub(in crate::nns) show_ranges: bool,
    pub(in crate::nns) verbose: bool,
    pub(in crate::nns) range_limit: usize,
    pub(in crate::nns) range_offset: usize,
}

///
/// CatalogInfoOptions
///
/// Parsed options accepted by `icq nns subnet info`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct CatalogInfoOptions {
    pub(in crate::nns) input: String,
    pub(in crate::nns) common: NnsCommonOptions,
    pub(in crate::nns) forced: Option<ResolveAs>,
}

impl CatalogListOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            common: NnsCommonOptions::from_matches(matches, network),
            filters: SubnetCatalogFilters {
                kind: typed_option(matches, "kind"),
                specialization: typed_option(matches, "specialization"),
                geographic_scope: typed_option(matches, "geo"),
            },
            show_ranges: matches.get_flag("show-ranges"),
            verbose: matches.get_flag("verbose"),
            range_limit: required_typed(matches, "range-limit"),
            range_offset: required_typed(matches, "range-offset"),
        }
    }
}

impl CatalogInfoOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            input: required_string(matches, "input"),
            common: NnsCommonOptions::from_matches(matches, network),
            forced: typed_option(matches, "as"),
        }
    }
}
