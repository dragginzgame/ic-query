use std::borrow::Cow;

/// Label a native SNS ballot code, preserving unknown codes for display.
pub(super) fn sns_proposal_vote_label(code: i32) -> Cow<'static, str> {
    match code {
        0 => Cow::Borrowed("unspecified"),
        1 => Cow::Borrowed("yes"),
        2 => Cow::Borrowed("no"),
        code => Cow::Owned(format!("unknown:{code}")),
    }
}

pub(super) const fn sns_treasury_kind(treasury: i32) -> &'static str {
    match treasury {
        0 => "unspecified",
        1 => "icp",
        2 => "sns_token",
        _ => "unknown",
    }
}

pub(super) const fn sns_swap_lifecycle_name(lifecycle: Option<i32>) -> Option<&'static str> {
    match lifecycle {
        None => None,
        Some(0) => Some("unspecified"),
        Some(1) => Some("pending"),
        Some(2) => Some("open"),
        Some(3) => Some("committed"),
        Some(4) => Some("aborted"),
        Some(5) => Some("adopted"),
        Some(_) => Some("unknown"),
    }
}

#[cfg(test)]
mod tests {
    use super::{sns_proposal_vote_label, sns_swap_lifecycle_name};

    #[test]
    fn lifecycle_names_preserve_native_codes_and_unknown_values() {
        for (code, expected) in [
            (None, None),
            (Some(0), Some("unspecified")),
            (Some(1), Some("pending")),
            (Some(2), Some("open")),
            (Some(3), Some("committed")),
            (Some(4), Some("aborted")),
            (Some(5), Some("adopted")),
            (Some(99), Some("unknown")),
        ] {
            assert_eq!(sns_swap_lifecycle_name(code), expected);
        }
    }
    #[test]
    fn proposal_vote_labels_preserve_known_and_unknown_codes() {
        for (code, label) in [
            (0, "unspecified"),
            (1, "yes"),
            (2, "no"),
            (99, "unknown:99"),
            (-1, "unknown:-1"),
        ] {
            assert_eq!(sns_proposal_vote_label(code), label);
        }
    }
}
