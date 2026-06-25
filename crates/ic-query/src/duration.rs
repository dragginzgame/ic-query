//! Module: duration
//!
//! Responsibility: parse CLI duration arguments and render compact duration values.
//!
//! Does not own: command definitions, cache policy decisions, or report layouts.
//!
//! Boundary: keeps human duration syntax centralized so commands and reports do not
//! each invent their own parsing or display conventions.

use thiserror::Error as ThisError;

///
/// DurationParseError
///
/// Error returned when a CLI duration value cannot be parsed into seconds.
///

#[derive(Debug, ThisError)]
pub enum DurationParseError {
    #[error("invalid duration {value:?}; use positive seconds or a value ending in s, m, h, or d")]
    Invalid { value: String },
}

/// Parses a positive duration string into seconds.
///
/// Accepts bare seconds or integer values ending in `s`, `m`, `h`, or `d`.
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

/// Renders seconds using the largest readable duration unit.
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
