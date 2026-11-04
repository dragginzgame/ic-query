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
