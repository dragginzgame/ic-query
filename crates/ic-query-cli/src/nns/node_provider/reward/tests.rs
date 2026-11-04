use super::*;
use crate::cli::clap::render_help;
use crate::cli::common::OutputFormat;

fn options<T>(command: ClapCommand, args: &[&str], from: impl FnOnce(&ArgMatches) -> T) -> T {
    let matches = crate::cli::parse_test_args(command, args).expect("valid options");
    from(&matches)
}

#[test]
fn help_discloses_authority_bounds_and_page_overlap() {
    let family = render_help(command());
    let list = render_help(list_command());
    let history = render_help(history_command());

    assert!(family.contains("Usage: icq nns node-provider reward [COMMAND]"));
    assert!(family.contains("history"));
    assert!(family.contains("info"));
    assert!(family.contains("list"));
    assert!(list.contains("Adjacent upstream offset pages can overlap"));
    assert!(list.contains("--max-reward-index"));
    assert!(history.contains("capped at 1000 requested observations"));
}

#[test]
fn options_preserve_explicit_bounds_and_raw_format() {
    let (format, list) = options(
        list_command(),
        &[
            "--limit",
            "25",
            "--offset",
            "50",
            "--max-reward-index",
            "6470",
            "--json",
        ],
        |matches| list_request(matches, 1_780_531_200),
    );
    assert_eq!(list.query.limit, 25);
    assert_eq!(list.query.offset, 50);
    assert_eq!(list.query.max_reward_index, Some(6_470));
    assert_eq!(format, OutputFormat::Json);

    let (_, history) = options(
        history_command(),
        &[
            "--start",
            "1752537600",
            "--end",
            "1784073600",
            "--step",
            "86400",
        ],
        |matches| history_request(matches, 1_780_531_200),
    );
    assert_eq!(history.query.start_unix_secs, 1_752_537_600);
    assert_eq!(history.query.end_unix_secs, 1_784_073_600);
    assert_eq!(history.query.step_secs, 86_400);
}
