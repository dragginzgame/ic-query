use crate::nns::leaf::NnsLeafCommandSpec;

const NODE_OPERATOR_LIST_HELP_AFTER: &str = "\
Examples:
  icq nns node-operator list
  icq nns node-operator list --verbose
  icq --network ic nns node-operator list --format json

Force-refresh cached native NNS data:
  icq nns node-operator refresh";
const NODE_OPERATOR_INFO_HELP_AFTER: &str = "\
Examples:
  icq nns node-operator info <node-operator>
  icq nns node-operator info <node-operator-prefix>
  icq --network ic nns node-operator info <node-operator> --format json

Force-refresh cached native NNS data:
  icq nns node-operator refresh";
const NODE_OPERATOR_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns node-operator refresh
  icq --network ic nns node-operator refresh --format json
  icq nns node-operator refresh --dry-run --output .icq/node-operator/ic/operators.preview.json";

pub(super) const NODE_OPERATOR_SPEC: NnsLeafCommandSpec = NnsLeafCommandSpec {
    command_name: "node-operator",
    bin_name: "icq nns node-operator",
    about: "Inspect NNS node-operator metadata",
    list_about: "List cached mainnet NNS node operators",
    info_about: "Show one cached mainnet NNS node operator",
    refresh_about: "Force-refresh and cache NNS node-operator metadata",
    list_help_after: NODE_OPERATOR_LIST_HELP_AFTER,
    info_help_after: NODE_OPERATOR_INFO_HELP_AFTER,
    refresh_help_after: NODE_OPERATOR_REFRESH_HELP_AFTER,
    input_value_name: "node-operator|node-operator-prefix",
    input_help: "Node-operator principal or unique node-operator principal prefix",
    list_source_help: "IC API endpoint used if the node-operator cache is missing",
    info_source_help: "IC API endpoint used if the node-operator cache is missing",
    refresh_source_help: "IC API endpoint used for native NNS registry queries",
    verbose_help: "Show full node-operator principals and registry metadata in text output",
    dry_run_help: "Fetch and validate without replacing the cached node-operator report",
    output_help: "Also write the fetched node-operator JSON to this path",
};
