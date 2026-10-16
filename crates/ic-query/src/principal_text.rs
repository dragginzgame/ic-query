//! Module: principal_text
//!
//! Responsibility: check strict canonical principal text in untrusted evidence.
//! Does not own: permissive request normalization or report-specific diagnostics.
//! Boundary: callers retain distinct parse and noncanonical error projections.

use candid::Principal;

/// Check principal spelling without normalizing the supplied evidence.
pub fn validate_canonical<E>(
    value: &str,
    invalid: impl FnOnce(String) -> E,
    noncanonical: impl FnOnce(String) -> E,
) -> Result<(), E> {
    let canonical = Principal::from_text(value)
        .map_err(|error| invalid(error.to_string()))?
        .to_text();
    if canonical == value {
        Ok(())
    } else {
        Err(noncanonical(canonical))
    }
}

#[cfg(test)]
mod tests {
    use super::validate_canonical;

    #[test]
    fn strict_evidence_admits_canonical_text_and_distinguishes_invalid_spelling() {
        for value in [
            "aaaaa-aa",
            "2vxsx-fae",
            "AAAAA-AA",
            "2VXSX-FAE",
            "aaaaaaa",
            " aaaaa-aa",
            "invalid",
            "é",
        ] {
            let result = validate_canonical(value, |_| None, Some);
            match value {
                "aaaaa-aa" | "2vxsx-fae" => assert_eq!(result, Ok(())),
                "AAAAA-AA" => assert_eq!(result, Err(Some("aaaaa-aa".to_string()))),
                "2VXSX-FAE" => assert_eq!(result, Err(Some("2vxsx-fae".to_string()))),
                _ => assert_eq!(result, Err(None)),
            }
        }
    }
}
