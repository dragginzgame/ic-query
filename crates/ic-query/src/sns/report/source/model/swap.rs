//! Module: sns::report::source::model::swap
//!
//! Responsibility: source result and invariants for bounded SNS swap collection.
//! Does not own: live transport, SNS lookup, report assembly, or rendering.
//! Boundary: validates target identity, native methods, partial gaps, and raw numeric values.

use crate::sns::report::{
    SnsHostError, SnsSwapComponent, SnsSwapDerivedState, SnsSwapLifecycle, SnsSwapQueryGap,
    SnsSwapSaleParameters,
};
use candid::Principal;
use std::collections::BTreeSet;

pub(in crate::sns::report) const SNS_SWAP_LIFECYCLE_METHOD: &str = "get_lifecycle";
pub(in crate::sns::report) const SNS_SWAP_SALE_PARAMETERS_METHOD: &str = "get_sale_parameters";
pub(in crate::sns::report) const SNS_SWAP_DERIVED_STATE_METHOD: &str = "get_derived_state";
pub(in crate::sns::report) const SNS_SWAP_QUERY_COUNT: usize = 3;

///
/// MainnetSnsSwap
///
/// Source-layer result from the three bounded native swap queries for one SNS.
///

#[derive(Clone, Debug, PartialEq)]
pub struct MainnetSnsSwap {
    /// Swap canister identity queried by the source.
    pub swap_canister_id: String,
    /// Native lifecycle query method.
    pub lifecycle_method: String,
    /// Native sale-parameters query method.
    pub sale_parameters_method: String,
    /// Native derived-state query method.
    pub derived_state_method: String,
    /// Whether the source can prove one point-in-time snapshot across all queries.
    pub point_in_time_guaranteed: bool,
    /// Lifecycle response, absent only when represented by a matching gap.
    pub lifecycle: Option<SnsSwapLifecycle>,
    /// Sale parameters, or `None` for either a matching gap or a successful empty response.
    pub sale_parameters: Option<SnsSwapSaleParameters>,
    /// Derived-state response, absent only when represented by a matching gap.
    pub derived_state: Option<SnsSwapDerivedState>,
    /// Component query failures retained by the source.
    pub gaps: Vec<SnsSwapQueryGap>,
}

pub(in crate::sns::report) fn canonicalize_mainnet_sns_swap(
    swap: &mut MainnetSnsSwap,
    expected_swap_canister_id: &str,
) -> Result<(), SnsHostError> {
    validate_principal("swap_canister_id", &swap.swap_canister_id)?;
    if swap.swap_canister_id != expected_swap_canister_id {
        return Err(invalid_swap(format!(
            "swap_canister_id is {:?}, expected {:?}",
            swap.swap_canister_id, expected_swap_canister_id
        )));
    }
    validate_exact(
        "lifecycle_method",
        SNS_SWAP_LIFECYCLE_METHOD,
        &swap.lifecycle_method,
    )?;
    validate_exact(
        "sale_parameters_method",
        SNS_SWAP_SALE_PARAMETERS_METHOD,
        &swap.sale_parameters_method,
    )?;
    validate_exact(
        "derived_state_method",
        SNS_SWAP_DERIVED_STATE_METHOD,
        &swap.derived_state_method,
    )?;
    if swap.point_in_time_guaranteed {
        return Err(invalid_swap(
            "three sequential swap queries cannot claim a point-in-time guarantee".to_string(),
        ));
    }

    if let Some(lifecycle) = &swap.lifecycle {
        let expected_name = sns_swap_lifecycle_name(lifecycle.lifecycle).map(str::to_string);
        if lifecycle.lifecycle_name != expected_name {
            return Err(invalid_swap(format!(
                "lifecycle_name is {:?}, expected {:?} for lifecycle {:?}",
                lifecycle.lifecycle_name, expected_name, lifecycle.lifecycle
            )));
        }
    }
    if let Some(rate) = swap
        .derived_state
        .as_ref()
        .and_then(|state| state.sns_tokens_per_icp)
        && (!rate.is_finite() || rate.is_sign_negative())
    {
        return Err(invalid_swap(format!(
            "sns_tokens_per_icp must be a finite non-negative value, got {rate}"
        )));
    }

    swap.gaps.sort_by_key(|gap| gap.component);
    let mut gap_components = BTreeSet::new();
    for gap in &swap.gaps {
        if !gap_components.insert(gap.component) {
            return Err(invalid_swap(format!(
                "duplicate {} query gap",
                gap.component.as_str()
            )));
        }
        let expected_method = sns_swap_component_method(gap.component);
        if gap.method != expected_method {
            return Err(invalid_swap(format!(
                "{} gap method is {:?}, expected {:?}",
                gap.component.as_str(),
                gap.method,
                expected_method
            )));
        }
        if gap.reason.trim().is_empty() {
            return Err(invalid_swap(format!(
                "{} query gap has an empty reason",
                gap.component.as_str()
            )));
        }
    }

    validate_component_result(
        SnsSwapComponent::Lifecycle,
        swap.lifecycle.is_some(),
        gap_components.contains(&SnsSwapComponent::Lifecycle),
        false,
    )?;
    validate_component_result(
        SnsSwapComponent::SaleParameters,
        swap.sale_parameters.is_some(),
        gap_components.contains(&SnsSwapComponent::SaleParameters),
        true,
    )?;
    validate_component_result(
        SnsSwapComponent::DerivedState,
        swap.derived_state.is_some(),
        gap_components.contains(&SnsSwapComponent::DerivedState),
        false,
    )?;
    Ok(())
}

pub(in crate::sns::report) const fn sns_swap_component_method(
    component: SnsSwapComponent,
) -> &'static str {
    match component {
        SnsSwapComponent::Lifecycle => SNS_SWAP_LIFECYCLE_METHOD,
        SnsSwapComponent::SaleParameters => SNS_SWAP_SALE_PARAMETERS_METHOD,
        SnsSwapComponent::DerivedState => SNS_SWAP_DERIVED_STATE_METHOD,
    }
}

pub(in crate::sns::report) const fn sns_swap_lifecycle_name(
    lifecycle: Option<i32>,
) -> Option<&'static str> {
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

fn validate_component_result(
    component: SnsSwapComponent,
    has_value: bool,
    has_gap: bool,
    empty_success_allowed: bool,
) -> Result<(), SnsHostError> {
    if has_value && has_gap {
        return Err(invalid_swap(format!(
            "{} has both a value and a query gap",
            component.as_str()
        )));
    }
    if !has_value && !has_gap && !empty_success_allowed {
        return Err(invalid_swap(format!(
            "{} has neither a value nor a query gap",
            component.as_str()
        )));
    }
    Ok(())
}

fn validate_exact(field: &'static str, expected: &str, actual: &str) -> Result<(), SnsHostError> {
    if actual != expected {
        return Err(invalid_swap(format!(
            "{field} is {actual:?}, expected {expected:?}"
        )));
    }
    Ok(())
}

fn validate_principal(field: &'static str, value: &str) -> Result<(), SnsHostError> {
    let principal = Principal::from_text(value)
        .map_err(|error| invalid_swap(format!("{field} {value:?} is invalid: {error}")))?;
    if principal.to_text() != value {
        return Err(invalid_swap(format!(
            "{field} {value:?} is not canonical principal text"
        )));
    }
    Ok(())
}

const fn invalid_swap(reason: String) -> SnsHostError {
    SnsHostError::InvalidSourceData {
        capability: "SNS swap",
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::sns_swap_lifecycle_name;

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
}
