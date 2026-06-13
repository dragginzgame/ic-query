use thiserror::Error as ThisError;

///
/// DurationParseError
///
#[derive(Debug, ThisError)]
pub enum DurationParseError {
    #[error("invalid duration {value:?}; use positive seconds or a value ending in s, m, h, or d")]
    Invalid { value: String },
}

pub fn parse_duration_seconds(value: &str) -> Result<u64, DurationParseError> {
    let (number, multiplier) = match value.as_bytes().last().copied() {
        Some(b's') => (&value[..value.len() - 1], 1),
        Some(b'm') => (&value[..value.len() - 1], 60),
        Some(b'h') => (&value[..value.len() - 1], 60 * 60),
        Some(b'd') => (&value[..value.len() - 1], 24 * 60 * 60),
        Some(b'0'..=b'9') => (value, 1),
        _ => {
            return Err(DurationParseError::Invalid {
                value: value.to_string(),
            });
        }
    };
    number
        .parse::<u64>()
        .ok()
        .and_then(|amount| amount.checked_mul(multiplier))
        .filter(|seconds| *seconds > 0)
        .ok_or_else(|| DurationParseError::Invalid {
            value: value.to_string(),
        })
}

#[must_use]
pub fn display_duration_seconds(seconds: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;

    if seconds == 0 {
        "0s".to_string()
    } else if seconds >= DAY {
        scaled_duration_unit_text(seconds, DAY, "d")
    } else if seconds >= HOUR {
        scaled_duration_unit_text(seconds, HOUR, "h")
    } else if seconds >= MINUTE {
        scaled_duration_unit_text(seconds, MINUTE, "m")
    } else {
        format!("{seconds}s")
    }
}

fn scaled_duration_unit_text(seconds: u64, unit_seconds: u64, suffix: &str) -> String {
    if seconds.is_multiple_of(unit_seconds) {
        return format!("{}{suffix}", seconds / unit_seconds);
    }
    let hundredths =
        ((u128::from(seconds) * 100) + (u128::from(unit_seconds) / 2)) / u128::from(unit_seconds);
    let whole = hundredths / 100;
    let fractional = hundredths % 100;
    format!("{whole}.{fractional:02}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::{display_duration_seconds, parse_duration_seconds};

    #[test]
    fn duration_display_uses_largest_readable_unit() {
        assert_eq!(display_duration_seconds(0), "0s");
        assert_eq!(display_duration_seconds(86_400), "1d");
        assert_eq!(display_duration_seconds(2_629_800), "30.44d");
        assert_eq!(display_duration_seconds(5_400), "1.50h");
        assert_eq!(display_duration_seconds(90), "1.50m");
        assert_eq!(display_duration_seconds(45), "45s");
    }

    #[test]
    fn duration_parser_accepts_integer_units() {
        assert_eq!(parse_duration_seconds("45").expect("seconds"), 45);
        assert_eq!(parse_duration_seconds("30m").expect("minutes"), 1_800);
        assert_eq!(parse_duration_seconds("2h").expect("hours"), 7_200);
        assert_eq!(parse_duration_seconds("1d").expect("days"), 86_400);
        assert!(parse_duration_seconds("0").is_err());
        assert!(parse_duration_seconds("1.5h").is_err());
    }
}
