//! Module: sns::report::source::model::swap
//!
//! Responsibility: source result and invariants for bounded SNS swap collection.
//! Does not own: live transport, SNS lookup, report assembly, or rendering.
//! Boundary: validates target identity, native methods, partial gaps, and raw numeric values.

use super::validation::SnsSourceValidator;
use crate::sns::report::{MainnetSnsSwap, SnsCanisterMethod, SnsHostError, SnsSwapComponent};
use std::collections::BTreeSet;

pub(in crate::sns::report) const SNS_SWAP_QUERY_COUNT: usize = 3;
const VALIDATOR: SnsSourceValidator = SnsSourceValidator::new("SNS swap");

pub(in crate::sns::report) fn canonicalize_mainnet_sns_swap(
    swap: &mut MainnetSnsSwap,
    expected_swap_canister_id: &str,
) -> Result<(), SnsHostError> {
    VALIDATOR.canonical_principal("swap_canister_id", &swap.swap_canister_id)?;
    if swap.swap_canister_id != expected_swap_canister_id {
        return Err(VALIDATOR.invalid(format!(
            "swap_canister_id is {:?}, expected {:?}",
            swap.swap_canister_id, expected_swap_canister_id
        )));
    }
    VALIDATOR.exact(
        "lifecycle_method",
        SnsCanisterMethod::GetLifecycle.as_str(),
        swap.lifecycle_method.as_str(),
    )?;
    VALIDATOR.exact(
        "sale_parameters_method",
        SnsCanisterMethod::GetSaleParameters.as_str(),
        swap.sale_parameters_method.as_str(),
    )?;
    VALIDATOR.exact(
        "derived_state_method",
        SnsCanisterMethod::GetDerivedState.as_str(),
        swap.derived_state_method.as_str(),
    )?;
    if swap.point_in_time_guaranteed {
        return Err(VALIDATOR.invalid(
            "three sequential swap queries cannot claim a point-in-time guarantee".to_string(),
        ));
    }

    if let Some(rate) = swap
        .derived_state
        .as_ref()
        .and_then(|state| state.sns_tokens_per_icp)
        && (!rate.is_finite() || rate.is_sign_negative())
    {
        return Err(VALIDATOR.invalid(format!(
            "sns_tokens_per_icp must be a finite non-negative value, got {rate}"
        )));
    }

    swap.gaps.sort_by_key(|gap| gap.component);
    let mut gap_components = BTreeSet::new();
    for gap in &swap.gaps {
        if !gap_components.insert(gap.component) {
            return Err(
                VALIDATOR.invalid(format!("duplicate {} query gap", gap.component.as_str()))
            );
        }
        let expected_method = sns_swap_component_method(gap.component);
        if gap.method != expected_method {
            return Err(VALIDATOR.invalid(format!(
                "{} gap method is {:?}, expected {:?}",
                gap.component.as_str(),
                gap.method.as_str(),
                expected_method.as_str()
            )));
        }
        if gap.reason.trim().is_empty() {
            return Err(VALIDATOR.invalid(format!(
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
) -> SnsCanisterMethod {
    match component {
        SnsSwapComponent::Lifecycle => SnsCanisterMethod::GetLifecycle,
        SnsSwapComponent::SaleParameters => SnsCanisterMethod::GetSaleParameters,
        SnsSwapComponent::DerivedState => SnsCanisterMethod::GetDerivedState,
    }
}

fn validate_component_result(
    component: SnsSwapComponent,
    has_value: bool,
    has_gap: bool,
    empty_success_allowed: bool,
) -> Result<(), SnsHostError> {
    if has_value && has_gap {
        return Err(VALIDATOR.invalid(format!(
            "{} has both a value and a query gap",
            component.as_str()
        )));
    }
    if !has_value && !has_gap && !empty_success_allowed {
        return Err(VALIDATOR.invalid(format!(
            "{} has neither a value nor a query gap",
            component.as_str()
        )));
    }
    Ok(())
}
