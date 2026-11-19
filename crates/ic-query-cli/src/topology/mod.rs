//! Command adapter for the topology library.

#[cfg(test)]
mod tests;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use ic_platform_types::Principal;
use std::process::ExitCode;

use crate::{CommandOutput, presentation};

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
                            .value_parser(value_parser!(ic_query_topology::cloud_engine::engine::EngineId)))
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
                .about("Inspect explicitly selected canister placement")
                .subcommand(
                    Command::new("placements")
                        .about("Read 1–32 selected placements within 60 seconds; partial output exits 1")
                        .arg(Arg::new("canisters").value_name("CANISTER_ID")
                            .required(true).num_args(1..=ic_query_topology::placement::collection::MAX_CANISTERS)
                            .value_parser(value_parser!(Principal)))
                        .arg(json_argument()),
                )
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
                .subcommand(Command::new("progress")
                    .about("Compare two authenticated state times; not uptime or maintenance completion")
                    .arg(Arg::new("subnet").required(true).value_name("SUBNET_ID").value_parser(value_parser!(Principal)))
                    .arg(interval_argument()).arg(json_argument()))
                .subcommand(Command::new("compare")
                    .about("Compare two selected placement candidates; partial evidence exits 1")
                    .arg(Arg::new("left").required(true).value_name("LEFT_SUBNET_ID").value_parser(value_parser!(Principal)))
                    .arg(Arg::new("right").required(true).value_name("RIGHT_SUBNET_ID").value_parser(value_parser!(Principal)))
                    .arg(Arg::new("overlap-details").long("overlap-details").action(ArgAction::SetTrue).conflicts_with("json").help("Show shared provider/data-center identities and per-side node counts"))
                    .arg(Arg::new("settings").long("settings").action(ArgAction::SetTrue).help("Add two independent Registry settings reads; total bound 200 seconds"))
                    .arg(json_argument()))
                .subcommand(Command::new("maintenance")
                    .about("Observe maintenance configuration and activity; not completion or uptime")
                    .arg(Arg::new("subnet").required(true).value_name("SUBNET_ID").value_parser(value_parser!(Principal)))
                    .arg(interval_argument())
                    .arg(Arg::new("expected-version").long("expected-version").help("Exact configured-version expectation; empty is meaningful"))
                    .arg(json_argument()))
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
                    Command::new("settings")
                        .about("Read registered features and resource limits; not effective capacity")
                        .arg(Arg::new("subnet").value_name("SUBNET_ID").required(true).value_parser(value_parser!(Principal)))
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
                .range(1..=i64::from(ic_query_topology::cloud_engine::available::MAX_PAGE_LIMIT)),
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
    if (entity, action) == ("subnet", "settings") {
        let id = *arguments
            .get_one::<Principal>("subnet")
            .expect("required subnet");
        let report = ic_query_topology::subnet::settings::fetch(id).await?;

        return Ok(settings_output(&report, arguments.get_flag("json"))?);
    }
    if (entity, action) == ("canister", "placements") {
        let ids: Vec<_> = arguments
            .get_many::<Principal>("canisters")
            .expect("required canister selection")
            .copied()
            .collect();
        let report = ic_query_topology::placement::collection::fetch(&ids).await?;

        return Ok(placements_output(&report, arguments.get_flag("json"))?);
    }
    if (entity, action) == ("cloud-engine", "prices") {
        let report = ic_query_topology::cloud_engine::prices::fetch().await?;

        return Ok(prices_output(&report, arguments.get_flag("json"))?);
    }
    if (entity, action) == ("cloud-engine", "available") {
        let page_limit = *arguments
            .get_one::<u16>("max-pages")
            .expect("default page budget");
        let report =
            ic_query_topology::cloud_engine::available::fetch_with_page_limit(page_limit).await?;

        return Ok(available_output(&report, arguments.get_flag("json"))?);
    }
    if (entity, action) == ("cloud-engine", "candidates") {
        let page_limit = *arguments
            .get_one::<u16>("max-pages")
            .expect("default page budget");
        let report = ic_query_topology::cloud_engine::candidates::fetch(page_limit).await?;

        return Ok(candidates_output(&report, arguments.get_flag("json"))?);
    }
    if (entity, action) == ("cloud-engine", "show") {
        let id = arguments
            .get_one::<ic_query_topology::cloud_engine::engine::EngineId>("engine")
            .expect("required typed engine ID")
            .clone();
        let report = ic_query_topology::cloud_engine::engine::fetch(id).await?;

        return Ok(engine_output(&report, arguments.get_flag("json"))?);
    }
    if (entity, action) == ("subnet", "nodes") {
        let id = *arguments
            .get_one::<Principal>("subnet")
            .expect("required typed argument");
        let report = ic_query_topology::subnet::nodes::fetch(id).await?;

        return Ok(nodes_output(&report, arguments.get_flag("json"))?);
    }
    if (entity, action) == ("subnet", "distribution") {
        let id = *arguments
            .get_one::<Principal>("subnet")
            .expect("required typed argument");
        let report = ic_query_topology::subnet::distribution::fetch(id).await?;

        return Ok(distribution_output(&report, arguments.get_flag("json"))?);
    }
    let text: Result<String, Box<dyn std::error::Error>> = match (entity, action) {
        ("subnet", "compare") => {
            if arguments.get_flag("settings") {
                let report = ic_query_topology::subnet::compare::fetch_with_settings(
                    *arguments
                        .get_one::<Principal>("left")
                        .expect("required left subnet"),
                    *arguments
                        .get_one::<Principal>("right")
                        .expect("required right subnet"),
                )
                .await?;

                return Ok(settings_comparison_output(
                    &report,
                    arguments.get_flag("json"),
                    arguments.get_flag("overlap-details"),
                )?);
            }
            let report = ic_query_topology::subnet::compare::fetch(
                *arguments
                    .get_one::<Principal>("left")
                    .expect("required left subnet"),
                *arguments
                    .get_one::<Principal>("right")
                    .expect("required right subnet"),
            )
            .await?;

            return Ok(compare_output(
                &report,
                arguments.get_flag("json"),
                arguments.get_flag("overlap-details"),
            )?);
        }
        ("subnet", "progress") => {
            let report = ic_query_topology::subnet::progress::fetch(
                *arguments
                    .get_one::<Principal>("subnet")
                    .expect("required subnet"),
                *arguments
                    .get_one::<u64>("interval-seconds")
                    .expect("default interval"),
            )
            .await?;

            return Ok(progress_output(&report, arguments.get_flag("json"))?);
        }
        ("subnet", "maintenance") => {
            let report = ic_query_topology::subnet::maintenance::fetch(
                *arguments
                    .get_one::<Principal>("subnet")
                    .expect("required subnet"),
                *arguments
                    .get_one::<u64>("interval-seconds")
                    .expect("default interval"),
                arguments.get_one::<String>("expected-version").cloned(),
            )
            .await?;

            return Ok(maintenance_output(&report, arguments.get_flag("json"))?);
        }
        ("node", "changes") => {
            let id = *arguments
                .get_one::<Principal>("node")
                .expect("required node");
            let interval = *arguments
                .get_one::<u64>("interval-seconds")
                .expect("default interval");
            let report = ic_query_topology::node::changes::fetch(id, interval).await?;

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
            let report = ic_query_topology::subnet::changes::fetch(id, interval).await?;

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
            let report = ic_query_topology::node::fetch(id).await?;

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
            let report = ic_query_topology::subnet::membership::fetch(id).await?;

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
            let report = ic_query_topology::subnet::configuration::fetch(id).await?;

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
            let report = ic_query_topology::subnet::activity::fetch(subnet_id, interval).await?;

            if arguments.get_flag("json") {
                Ok(serde_json::to_string_pretty(&report)?)
            } else {
                Ok(presentation::subnet_activity(&report))
            }
        }
        ("subnet", "list") => {
            let report = ic_query_topology::subnet::list::fetch().await?;

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
            let report = ic_query_topology::placement::fetch(id).await?;

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
            let report = ic_query_topology::subnet::fetch(id).await?;

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
            let report = ic_query_topology::subnet::metrics::fetch(id).await?;

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
    report: &ic_query_topology::subnet::distribution::SubnetDistributionReport,
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

fn maintenance_output(
    report: &ic_query_topology::subnet::maintenance::MaintenanceReport,
    json: bool,
) -> Result<CommandOutput, serde_json::Error> {
    Ok(CommandOutput {
        text: if json {
            serde_json::to_string_pretty(report)?
        } else {
            presentation::maintenance::render(report)
        },
        exit_code: if report.comparisons_available() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}

fn engine_output(
    report: &ic_query_topology::cloud_engine::engine::EngineReport,
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
    report: &ic_query_topology::cloud_engine::candidates::CandidatesReport,
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
    report: &ic_query_topology::cloud_engine::available::AvailableNodesReport,
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
    report: &ic_query_topology::cloud_engine::prices::CloudEnginePricesReport,
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
    report: &ic_query_topology::subnet::nodes::SubnetNodesReport,
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

fn compare_output(
    report: &ic_query_topology::subnet::compare::SubnetComparisonReport,
    json: bool,
    overlap_details: bool,
) -> Result<CommandOutput, serde_json::Error> {
    Ok(CommandOutput {
        text: if json {
            serde_json::to_string_pretty(report)?
        } else {
            presentation::compare::render(report, overlap_details)
        },
        exit_code: if report.collection_complete {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}

fn placements_output(
    report: &ic_query_topology::placement::collection::PlacementCollectionReport,
    json: bool,
) -> Result<CommandOutput, serde_json::Error> {
    Ok(CommandOutput {
        text: if json {
            serde_json::to_string_pretty(report)?
        } else {
            presentation::placements::render(report)
        },
        exit_code: if report.collection_complete {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}

fn settings_output(
    report: &ic_query_topology::subnet::settings::SubnetSettingsReport,
    json: bool,
) -> Result<CommandOutput, serde_json::Error> {
    Ok(CommandOutput {
        text: if json {
            serde_json::to_string_pretty(report)?
        } else {
            presentation::settings::render(report)
        },
        exit_code: ExitCode::SUCCESS,
    })
}

fn progress_output(
    report: &ic_query_topology::subnet::progress::SubnetProgressReport,
    json: bool,
) -> Result<CommandOutput, serde_json::Error> {
    Ok(CommandOutput {
        text: if json {
            serde_json::to_string_pretty(report)?
        } else {
            presentation::progress::render(report)
        },
        exit_code: if report.collection_complete {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}

fn settings_comparison_output(
    report: &ic_query_topology::subnet::compare::SettingsComparisonReport,
    json: bool,
    overlap_details: bool,
) -> Result<CommandOutput, serde_json::Error> {
    Ok(CommandOutput {
        text: if json {
            serde_json::to_string_pretty(report)?
        } else {
            format!(
                "Combined collection: {} | total elapsed: {:.2}s\n\nBase comparison\n{}{}",
                if report.collection_complete {
                    "complete"
                } else {
                    "partial"
                },
                report.elapsed.as_secs_f64(),
                presentation::compare::render(&report.comparison, overlap_details),
                presentation::settings::comparison(report)
            )
        },
        exit_code: if report.collection_complete {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}
