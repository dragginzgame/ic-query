//! Command adapter for the topology library.

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use ic_platform_types::Principal;
use std::process::ExitCode;

use crate::presentation;

pub(crate) struct CommandOutput {
    pub(crate) text: String,
    pub(crate) exit_code: ExitCode,
}

pub(crate) fn command() -> Command {
    Command::new("topology")
        .about("Inspect infrastructure evidence for Canic placement")
        .subcommand(
            Command::new("cloud-engine")
                .about("Inspect public cloud-engine topology, prices and candidate nodes")
                .subcommand(
                    Command::new("show")
                        .about("Read one public engine's subnet/operator and node lists; not liveness")
                        .arg(Arg::new("engine").value_name("ENGINE_ID").required(true)
                            .value_parser(value_parser!(ic_topology::cloud_engine::engine::EngineId)))
                        .arg(json_argument()),
                )
                .subcommand(
                    Command::new("candidates")
                        .about("Match candidates to public price declarations; does not select a tariff")
                        .arg(page_limit_argument())
                        .arg(json_argument()),
                )
                .subcommand(
                    Command::new("available")
                        .about("List public candidate nodes with bounded pagination; partial output exits 1")
                        .arg(page_limit_argument())
                        .arg(json_argument()),
                )
                .subcommand(
                    Command::new("prices")
                        .about("Read daily marketplace prices and network markup; partial output exits 1")
                        .arg(json_argument()),
                ),
        )
        .subcommand(
            Command::new("canister")
                .about("Inspect one canister's topology")
                .subcommand(
                    Command::new("show")
                        .about("Look up one canister's subnet in the mainnet NNS Registry")
                        .arg(
                            Arg::new("canister")
                                .value_name("CANISTER_ID")
                                .required(true)
                                .value_parser(value_parser!(Principal)),
                        )
                        .arg(json_argument()),
                ),
        )
        .subcommand(
            Command::new("node")
                .about("Inspect one node's registered configuration")
                .subcommand(
                    Command::new("changes")
                        .about("Compare registered chip, HostOS and operator changes; not TEE readiness")
                        .arg(
                            Arg::new("node")
                                .value_name("NODE_ID")
                                .required(true)
                                .value_parser(value_parser!(Principal)),
                        )
                        .arg(interval_argument())
                        .arg(json_argument()),
                )
                .subcommand(
                    Command::new("show")
                        .about("Read Registry operator, chip ID and configured HostOS; not verified attestation")
                        .arg(
                            Arg::new("node")
                                .value_name("NODE_ID")
                                .required(true)
                                .value_parser(value_parser!(Principal)),
                        )
                        .arg(json_argument()),
                ),
        )
        .subcommand(
            Command::new("subnet")
                .about("Inspect subnet topology")
                .subcommand(
                    Command::new("activity")
                        .about("Compare two certified message totals; not uptime or throughput")
                        .arg(
                            Arg::new("subnet")
                                .value_name("SUBNET_ID")
                                .required(true)
                                .value_parser(value_parser!(Principal)),
                        )
                        .arg(interval_argument())
                        .arg(json_argument()),
                )
                .subcommand(
                    Command::new("changes")
                        .about("Compare two Registry configurations; not a maintenance event log")
                        .arg(
                            Arg::new("subnet")
                                .value_name("SUBNET_ID")
                                .required(true)
                                .value_parser(value_parser!(Principal)),
                        )
                        .arg(interval_argument())
                        .arg(json_argument()),
                )
                .subcommand(
                    Command::new("configuration")
                        .about("Read Registry membership, configured version and halt flags; not running state")
                        .arg(
                            Arg::new("subnet")
                                .value_name("SUBNET_ID")
                                .required(true)
                                .value_parser(value_parser!(Principal)),
                        )
                        .arg(json_argument()),
                )
                .subcommand(
                    Command::new("distribution")
                        .about("Read providers and registered locations at one Registry version; partial output exits 1")
                        .arg(
                            Arg::new("subnet")
                                .value_name("SUBNET_ID")
                                .required(true)
                                .value_parser(value_parser!(Principal)),
                        )
                        .arg(json_argument()),
                )
                .subcommand(
                    Command::new("list")
                        .about("List registered subnet IDs from the mainnet NNS Registry")
                        .arg(json_argument()),
                )
                .subcommand(
                    Command::new("membership")
                        .about("Compare registered members and observed node keys; not an atomic snapshot")
                        .arg(
                            Arg::new("subnet")
                                .value_name("SUBNET_ID")
                                .required(true)
                                .value_parser(value_parser!(Principal)),
                        )
                        .arg(json_argument()),
                )
                .subcommand(
                    Command::new("metrics")
                        .about(
                            "Read certified subnet resource values; not capacity, rates or uptime",
                        )
                        .arg(
                            Arg::new("subnet")
                                .value_name("SUBNET_ID")
                                .required(true)
                                .value_parser(value_parser!(Principal)),
                        )
                        .arg(json_argument()),
                )
                .subcommand(
                    Command::new("nodes")
                        .about("Read registered nodes for one subnet; partial output exits with status 1")
                        .arg(
                            Arg::new("subnet")
                                .value_name("SUBNET_ID")
                                .required(true)
                                .value_parser(value_parser!(Principal)),
                        )
                        .arg(json_argument()),
                )
                .subcommand(
                    Command::new("show")
                        .about("Observe subnet type and node identities in certified state")
                        .arg(
                            Arg::new("subnet")
                                .value_name("SUBNET_ID")
                                .required(true)
                                .value_parser(value_parser!(Principal)),
                        )
                        .arg(json_argument()),
                ),
        )
}

fn page_limit_argument() -> Arg {
    Arg::new("max-pages")
        .long("max-pages")
        .help("Read at most 1–100 pages of up to 100 nodes within 60 seconds")
        .default_value("10")
        .value_parser(
            value_parser!(u16)
                .range(1..=i64::from(ic_topology::cloud_engine::available::MAX_PAGE_LIMIT)),
        )
}

fn json_argument() -> Arg {
    Arg::new("json")
        .long("json")
        .help("Print the report and its coverage as JSON")
        .action(ArgAction::SetTrue)
}

fn interval_argument() -> Arg {
    Arg::new("interval-seconds")
        .long("interval-seconds")
        .help("Wait 1–60 seconds after the first read before starting the second")
        .default_value("10")
        .value_parser(value_parser!(u64))
}

pub(crate) async fn run(matches: &ArgMatches) -> Result<CommandOutput, Box<dyn std::error::Error>> {
    let (entity, scope) = matches
        .subcommand()
        .expect("entrypoint handles namespace help");
    let (action, arguments) = scope
        .subcommand()
        .expect("entrypoint handles namespace help");
    if (entity, action) == ("cloud-engine", "prices") {
        let report = ic_topology::cloud_engine::prices::fetch().await?;
        return Ok(prices_output(&report, arguments.get_flag("json"))?);
    }
    if (entity, action) == ("cloud-engine", "available") {
        let page_limit = *arguments
            .get_one::<u16>("max-pages")
            .expect("default page budget");
        let report =
            ic_topology::cloud_engine::available::fetch_with_page_limit(page_limit).await?;
        return Ok(available_output(&report, arguments.get_flag("json"))?);
    }
    if (entity, action) == ("cloud-engine", "candidates") {
        let page_limit = *arguments
            .get_one::<u16>("max-pages")
            .expect("default page budget");
        let report = ic_topology::cloud_engine::candidates::fetch(page_limit).await?;
        return Ok(candidates_output(&report, arguments.get_flag("json"))?);
    }
    if (entity, action) == ("cloud-engine", "show") {
        let id = arguments
            .get_one::<ic_topology::cloud_engine::engine::EngineId>("engine")
            .expect("required typed engine ID")
            .clone();
        let report = ic_topology::cloud_engine::engine::fetch(id).await?;
        return Ok(engine_output(&report, arguments.get_flag("json"))?);
    }
    if (entity, action) == ("subnet", "nodes") {
        let id = *arguments
            .get_one::<Principal>("subnet")
            .expect("required typed argument");
        let report = ic_topology::subnet::nodes::fetch(id).await?;
        return Ok(nodes_output(&report, arguments.get_flag("json"))?);
    }
    if (entity, action) == ("subnet", "distribution") {
        let id = *arguments
            .get_one::<Principal>("subnet")
            .expect("required typed argument");
        let report = ic_topology::subnet::distribution::fetch(id).await?;
        return Ok(distribution_output(&report, arguments.get_flag("json"))?);
    }
    let text: Result<String, Box<dyn std::error::Error>> = match (entity, action) {
        ("node", "changes") => {
            let id = *arguments
                .get_one::<Principal>("node")
                .expect("required node");
            let interval = *arguments
                .get_one::<u64>("interval-seconds")
                .expect("default interval");
            let report = ic_topology::node::changes::fetch(id, interval).await?;
            if arguments.get_flag("json") {
                Ok(serde_json::to_string_pretty(&report)?)
            } else {
                Ok(presentation::node_changes(&report))
            }
        }
        ("subnet", "changes") => {
            let id = *arguments
                .get_one::<Principal>("subnet")
                .expect("required typed argument");
            let interval = *arguments
                .get_one::<u64>("interval-seconds")
                .expect("default interval");
            let report = ic_topology::subnet::changes::fetch(id, interval).await?;
            if arguments.get_flag("json") {
                Ok(serde_json::to_string_pretty(&report)?)
            } else {
                Ok(presentation::subnet_changes(&report))
            }
        }
        ("node", "show") => {
            let id = *arguments
                .get_one::<Principal>("node")
                .expect("required typed argument");
            let report = ic_topology::node::fetch(id).await?;
            if arguments.get_flag("json") {
                Ok(serde_json::to_string_pretty(&report)?)
            } else {
                Ok(presentation::node(&report))
            }
        }
        ("subnet", "membership") => {
            let id = *arguments
                .get_one::<Principal>("subnet")
                .expect("required typed argument");
            let report = ic_topology::subnet::membership::fetch(id).await?;
            if arguments.get_flag("json") {
                Ok(serde_json::to_string_pretty(&report)?)
            } else {
                Ok(presentation::subnet_membership(&report))
            }
        }
        ("subnet", "configuration") => {
            let id = *arguments
                .get_one::<Principal>("subnet")
                .expect("required typed argument");
            let report = ic_topology::subnet::configuration::fetch(id).await?;
            if arguments.get_flag("json") {
                Ok(serde_json::to_string_pretty(&report)?)
            } else {
                Ok(presentation::subnet_configuration(&report))
            }
        }
        ("subnet", "activity") => {
            let subnet_id = *arguments
                .get_one::<Principal>("subnet")
                .expect("required subnet");
            let interval = *arguments
                .get_one::<u64>("interval-seconds")
                .expect("default interval");
            let report = ic_topology::subnet::activity::fetch(subnet_id, interval).await?;
            if arguments.get_flag("json") {
                Ok(serde_json::to_string_pretty(&report)?)
            } else {
                Ok(presentation::subnet_activity(&report))
            }
        }
        ("subnet", "list") => {
            let report = ic_topology::subnet::list::fetch().await?;
            if arguments.get_flag("json") {
                Ok(serde_json::to_string_pretty(&report)?)
            } else {
                Ok(presentation::subnet_list(&report))
            }
        }
        ("canister", "show") => {
            let id = *arguments
                .get_one::<Principal>("canister")
                .expect("required typed argument");
            let report = ic_topology::placement::fetch(id).await?;
            if arguments.get_flag("json") {
                Ok(serde_json::to_string_pretty(&report)?)
            } else {
                Ok(presentation::placement(&report))
            }
        }
        ("subnet", "show") => {
            let id = *arguments
                .get_one::<Principal>("subnet")
                .expect("required typed argument");
            let report = ic_topology::subnet::fetch(id).await?;
            if arguments.get_flag("json") {
                Ok(serde_json::to_string_pretty(&report)?)
            } else {
                Ok(presentation::subnet(&report))
            }
        }
        ("subnet", "metrics") => {
            let id = *arguments
                .get_one::<Principal>("subnet")
                .expect("required typed argument");
            let report = ic_topology::subnet::metrics::fetch(id).await?;
            if arguments.get_flag("json") {
                Ok(serde_json::to_string_pretty(&report)?)
            } else {
                Ok(presentation::subnet_metrics(&report))
            }
        }
        _ => unreachable!(
            "Clap admits only declared operations; the entrypoint handles namespace help"
        ),
    };
    Ok(CommandOutput {
        text: text?,
        exit_code: ExitCode::SUCCESS,
    })
}

fn distribution_output(
    report: &ic_topology::subnet::distribution::SubnetDistributionReport,
    json: bool,
) -> Result<CommandOutput, serde_json::Error> {
    Ok(CommandOutput {
        text: if json {
            serde_json::to_string_pretty(report)?
        } else {
            presentation::subnet_distribution(report)
        },
        exit_code: if report.complete {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}

fn engine_output(
    report: &ic_topology::cloud_engine::engine::EngineReport,
    json: bool,
) -> Result<CommandOutput, serde_json::Error> {
    Ok(CommandOutput {
        text: if json {
            serde_json::to_string_pretty(report)?
        } else {
            presentation::cloud_engine(report)
        },
        exit_code: ExitCode::SUCCESS,
    })
}

fn candidates_output(
    report: &ic_topology::cloud_engine::candidates::CandidatesReport,
    json: bool,
) -> Result<CommandOutput, serde_json::Error> {
    Ok(CommandOutput {
        text: if json {
            serde_json::to_string_pretty(report)?
        } else {
            presentation::cloud_engine_candidates(report)
        },
        exit_code: if report.collection_complete {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}

fn available_output(
    report: &ic_topology::cloud_engine::available::AvailableNodesReport,
    json: bool,
) -> Result<CommandOutput, serde_json::Error> {
    Ok(CommandOutput {
        text: if json {
            serde_json::to_string_pretty(report)?
        } else {
            presentation::cloud_engine_available(report)
        },
        exit_code: if report.pagination_exhausted {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}

fn prices_output(
    report: &ic_topology::cloud_engine::prices::CloudEnginePricesReport,
    json: bool,
) -> Result<CommandOutput, serde_json::Error> {
    Ok(CommandOutput {
        text: if json {
            serde_json::to_string_pretty(report)?
        } else {
            presentation::cloud_engine_prices(report)
        },
        exit_code: if report.complete {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}

fn nodes_output(
    report: &ic_topology::subnet::nodes::SubnetNodesReport,
    json: bool,
) -> Result<CommandOutput, serde_json::Error> {
    Ok(CommandOutput {
        text: if json {
            serde_json::to_string_pretty(report)?
        } else {
            presentation::subnet_nodes(report)
        },
        exit_code: if report.all_node_reads_succeeded {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}

#[cfg(test)]
mod tests;
