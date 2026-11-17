//! Pure, escaped human views of migration evidence; JSON keeps raw fields.
use ic_host_text::bordered_table;
use ic_query_sns::{
    Observation, Outcome,
    balance::BalanceReport,
    neuron::{Neuron, NeuronReport, NeuronsReport},
};

fn value<T: ToString>(o: &Observation<T>) -> String {
    match &o.outcome {
        Outcome::Read { value } => value.to_string(),
        Outcome::Failed { error } => format!("unavailable: {error}"),
    }
}
fn time<T>(o: &Observation<T>) -> String {
    format!(
        "{} – {}",
        o.started_at_unix_ms
            .map_or_else(|| "unavailable".into(), |v| v.to_string()),
        o.finished_at_unix_ms
            .map_or_else(|| "unavailable".into(), |v| v.to_string())
    )
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
const NOTICE: &str = "Successful queries verify node signatures, not certified application state. Separate reads are not an atomic snapshot. No migration or entitlement decision is made. Use --json for raw fields and failures.";
pub(crate) fn balance(report: &BalanceReport) -> String {
    let rows = vec![
        vec![
            "Balance (base units)".into(),
            value(&report.balance),
            time(&report.balance),
        ],
        vec![
            "Decimals".into(),
            value(&report.decimals),
            time(&report.decimals),
        ],
        vec!["Symbol".into(), value(&report.symbol), time(&report.symbol)],
    ];
    format!(
        "Ledger: {}\nOwner: {}\nSubaccount: {}\nSource: {}\nComplete: {}\n{}\n{}",
        report.ledger,
        report.account.owner,
        report
            .account
            .subaccount
            .as_ref()
            .map_or_else(|| "absent (default)".into(), |s| hex(s)),
        report.endpoint,
        report.complete,
        bordered_table(&["FIELD", "VALUE", "LOCAL UNIX MS WINDOW"], &rows),
        NOTICE
    )
}
fn row(neuron: &Neuron) -> Vec<String> {
    vec![
        neuron
            .id
            .as_ref()
            .map_or_else(|| "absent".into(), |id| hex(&id.id)),
        neuron.cached_neuron_stake_e8s.to_string(),
        neuron.neuron_fees_e8s.to_string(),
        neuron.maturity_e8s_equivalent.to_string(),
        neuron
            .staked_maturity_e8s_equivalent
            .map_or_else(|| "absent".into(), |v| v.to_string()),
    ]
}
fn table(rows: &[Vec<String>]) -> String {
    bordered_table(
        &[
            "NEURON ID",
            "CACHED STAKE E8S",
            "FEES E8S",
            "MATURITY E8S EQUIV",
            "STAKED MATURITY E8S EQUIV",
        ],
        rows,
    )
}
pub(crate) fn neuron(report: &NeuronReport) -> String {
    let rows = match &report.neuron.outcome {
        Outcome::Read { value } => vec![row(value)],
        Outcome::Failed { .. } => vec![],
    };
    let failure = match &report.neuron.outcome {
        Outcome::Read { .. } => String::new(),
        Outcome::Failed { error } => bordered_table(&["FAILURE"], &[vec![error.to_string()]]),
    };
    format!(
        "Governance: {}\nRequested neuron: {}\nSource: {} get_neuron\nLocal Unix ms: {}\n{}\n{}\nSelected fields only; permissions, dissolve state and other raw fields are in --json.\n{}",
        report.governance,
        hex(&report.neuron_id),
        report.endpoint,
        time(&report.neuron),
        table(&rows),
        failure,
        NOTICE
    )
}
pub(crate) fn neurons(report: &NeuronsReport) -> String {
    let mut rows = Vec::new();
    let mut pages = Vec::new();
    for (index, page) in report.pages.iter().enumerate() {
        let status = match &page.observation.outcome {
            Outcome::Read { value } => {
                rows.extend(value.iter().map(row));
                format!("{} records", value.len())
            }
            Outcome::Failed { error } => format!("unavailable: {error}"),
        };
        pages.push(vec![
            (index + 1).to_string(),
            time(&page.observation),
            status,
        ]);
    }
    format!(
        "Governance: {}\nSource: {} list_neurons\nPermission principal: {}\nStart after: {}\nRetained: {} | traversal exhausted: {} | stop: {:?}\n{}\n{}\nSelected fields only; use --json for all retained fields and cursors.\n{}",
        report.request.governance,
        report.endpoint,
        report
            .request
            .of_principal
            .map_or_else(|| "unfiltered".into(), |p| p.to_string()),
        report
            .request
            .start_after
            .as_ref()
            .map_or_else(|| "beginning".into(), |id| hex(id)),
        report.retained_neurons,
        report.traversal_exhausted,
        report.stop_reason,
        table(&rows),
        bordered_table(&["PAGE", "LOCAL UNIX MS WINDOW", "OUTCOME"], &pages),
        NOTICE
    )
}

#[cfg(test)]
mod tests;
