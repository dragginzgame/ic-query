pub(super) const TOPOLOGY_SUMMARY_HELP_AFTER: &str = "\
Examples:
  icq nns topology summary
  icq --network ic nns topology summary --format json
  icq nns topology summary --source-endpoint https://icp-api.io";

pub(super) const TOPOLOGY_COVERAGE_HELP_AFTER: &str = "\
Examples:
  icq nns topology coverage
  icq --network ic nns topology coverage --format json
  icq nns topology coverage --source-endpoint https://icp-api.io";

pub(super) const TOPOLOGY_VERSIONS_HELP_AFTER: &str = "\
Examples:
  icq nns topology versions
  icq --network ic nns topology versions --format json
  icq nns topology versions --source-endpoint https://icp-api.io";

pub(super) const TOPOLOGY_HEALTH_HELP_AFTER: &str = "\
Examples:
  icq nns topology health
  icq --network ic nns topology health --format json
  icq nns topology health --source-endpoint https://icp-api.io";

pub(super) const TOPOLOGY_GAPS_HELP_AFTER: &str = "\
Examples:
  icq nns topology gaps
  icq --network ic nns topology gaps --format json
  icq nns topology gaps --source-endpoint https://icp-api.io";

pub(super) const TOPOLOGY_CAPACITY_HELP_AFTER: &str = "\
Examples:
  icq nns topology capacity
  icq --network ic nns topology capacity --format json
  icq nns topology capacity --source-endpoint https://icp-api.io";

pub(super) const TOPOLOGY_REGIONS_HELP_AFTER: &str = "\
Examples:
  icq nns topology regions
  icq --network ic nns topology regions --format json
  icq nns topology regions --source-endpoint https://icp-api.io";

pub(super) const TOPOLOGY_PROVIDERS_HELP_AFTER: &str = "\
Examples:
  icq nns topology providers
  icq --network ic nns topology providers --format json
  icq nns topology providers --source-endpoint https://icp-api.io";

pub(super) const TOPOLOGY_COMPONENT_CACHE_SOURCE_HELP: &str =
    "IC API endpoint used if a topology component cache is missing";
pub(super) const TOPOLOGY_OPERATOR_CACHE_SOURCE_HELP: &str =
    "IC API endpoint used if the node-operator cache is missing";
pub(super) const TOPOLOGY_DATA_CENTER_CACHE_SOURCE_HELP: &str =
    "IC API endpoint used if the data-center cache is missing";
