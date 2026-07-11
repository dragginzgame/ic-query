//! Module: text_value
//!
//! Responsibility: make untrusted scalar values safe for terminal text output.
//! Does not own: JSON serialization, report layout, or table alignment.
//! Boundary: escapes control characters while preserving printable Unicode.

use std::fmt::Write;

pub fn sanitize_text(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\n' => sanitized.push_str("\\n"),
            '\r' => sanitized.push_str("\\r"),
            '\t' => sanitized.push_str("\\t"),
            character if character.is_control() => {
                write!(sanitized, "\\u{{{:x}}}", u32::from(character))
                    .expect("writing to a String cannot fail");
            }
            character => sanitized.push(character),
        }
    }
    sanitized
}

pub fn truncate_text(value: &str, limit: usize) -> String {
    let value = sanitize_text(value);
    if value.chars().count() <= limit {
        return value;
    }
    let mut truncated = value.chars().take(limit).collect::<String>();
    truncated.push_str("...");
    truncated
}

pub fn optional_u64_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

pub const fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
