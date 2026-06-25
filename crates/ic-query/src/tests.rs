#[cfg(feature = "host")]
use crate::{
    duration::{display_duration_seconds, parse_duration_seconds},
    token_amount::e8s_decimal_text,
};
use crate::{
    table::{ColumnAlign, render_table},
    token_amount::base_units_decimal_text,
};

#[test]
fn base_units_render_as_two_decimal_token_amounts() {
    assert_eq!(base_units_decimal_text("0", 8), "0.00");
    assert_eq!(base_units_decimal_text("000000000", 8), "0.00");
    assert_eq!(base_units_decimal_text("10_000", 8), "0.00");
    assert_eq!(
        base_units_decimal_text("100_923_109_141_460", 8),
        "1009231.09"
    );
    assert_eq!(base_units_decimal_text("500000", 8), "0.01");
    assert_eq!(base_units_decimal_text("123456789", 8), "1.23");
    assert_eq!(base_units_decimal_text("123500000", 8), "1.24");
    assert_eq!(base_units_decimal_text("3000000000000", 8), "30000.00");
    assert_eq!(base_units_decimal_text("123", 0), "123.00");
    assert_eq!(base_units_decimal_text("123", 1), "12.30");
    assert_eq!(base_units_decimal_text("123", 2), "1.23");
    assert_eq!(base_units_decimal_text("999", 3), "1.00");
    assert_eq!(base_units_decimal_text("not-a-number", 8), "not-a-number");
}

#[test]
#[cfg(feature = "host")]
fn e8s_render_as_two_decimal_token_amounts() {
    assert_eq!(e8s_decimal_text(0), "0.00");
    assert_eq!(e8s_decimal_text(123), "0.00");
    assert_eq!(e8s_decimal_text(499_999), "0.00");
    assert_eq!(e8s_decimal_text(500_000), "0.01");
    assert_eq!(e8s_decimal_text(100_000_000), "1.00");
    assert_eq!(e8s_decimal_text(123_456_789), "1.23");
    assert_eq!(e8s_decimal_text(123_500_000), "1.24");
    assert_eq!(e8s_decimal_text(3_000_000_000_000), "30000.00");
}

#[test]
fn render_table_handles_long_left_aligned_cells() {
    let rows = [[
        "ICRC-1".to_string(),
        "https://github.com/dfinity/ICRC-1?with=a-long-token-metadata-url".to_string(),
    ]];

    let table = render_table(
        &["STANDARD", "URL"],
        &rows,
        &[ColumnAlign::Left, ColumnAlign::Left],
    );

    assert!(table.contains("ICRC-1"));
    assert!(table.contains("a-long-token-metadata-url"));
}

#[test]
fn render_table_right_aligns_cells() {
    let rows = [["1".to_string(), "Dragginz".to_string()]];

    let table = render_table(
        &["ID", "NAME"],
        &rows,
        &[ColumnAlign::Right, ColumnAlign::Left],
    );

    assert!(table.contains("ID   NAME"));
    assert!(table.contains(" 1   Dragginz"));
}

#[test]
#[cfg(feature = "host")]
fn duration_display_uses_largest_readable_unit() {
    assert_eq!(display_duration_seconds(0), "0s");
    assert_eq!(display_duration_seconds(86_400), "1d");
    assert_eq!(display_duration_seconds(2_629_800), "30.44d");
    assert_eq!(display_duration_seconds(5_400), "1.50h");
    assert_eq!(display_duration_seconds(90), "1.50m");
    assert_eq!(display_duration_seconds(45), "45s");
}

#[test]
#[cfg(feature = "host")]
fn duration_parser_accepts_integer_units() {
    assert_eq!(parse_duration_seconds("45").expect("seconds"), 45);
    assert_eq!(parse_duration_seconds("30m").expect("minutes"), 1_800);
    assert_eq!(parse_duration_seconds("2h").expect("hours"), 7_200);
    assert_eq!(parse_duration_seconds("1d").expect("days"), 86_400);
    assert!(parse_duration_seconds("0").is_err());
    assert!(parse_duration_seconds("1.5h").is_err());
}
