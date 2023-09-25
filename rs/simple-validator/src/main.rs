mod access;
mod lifecycle;
mod memory;
mod types;

use crate::access::*;
use crate::memory::*;
use crate::types::*;

use candid::{decode_one, encode_args, encode_one, Principal};
use ic_cdk_macros::{query, update};

/// The governance canister may notify this canister of a new proposal.
/// This canister will validate the proposal and call `validate` of the governance canister.
#[update]
pub async fn notify_validator(proposal_id: Index, payload: ProposalPayload) {
    assert_eq!(ic_cdk::api::caller(), get_governance());
    let _ = validate_and_respond(proposal_id, &payload).await;
}

#[update]
pub async fn sync_with_governance() -> Result<(), ReturnError> {
    let config = memory::get_config()?;
    let res = ic_cdk::api::call::call_raw128(
        config.governance_canister,
        "get_all_submitted_proposal_ids",
        &encode_one(()).unwrap(),
        0,
    )
    .await
    .map_err(|_| ReturnError::InterCanisterCallError)?;
    let result: Vec<Index> = decode_one(&res).unwrap();

    // Set PropsalState for each non-existing proposal
    for index in result {
        if get_proposal_validation(index).is_none() {
            let res = ic_cdk::api::call::call_raw128(
                config.governance_canister,
                "get_proposal_payload",
                &encode_one(index).unwrap(),
                0,
            )
            .await
            .map_err(|_| ReturnError::InterCanisterCallError)?;
            let result: Option<ProposalPayload> = decode_one(&res).unwrap();

            if let Some(payload) = result {
                let validated = validate_and_respond(index, &payload).await;
                add_proposal_validation(index, validated);
            }
        }
    }

    Ok(())
}

pub async fn validate_and_respond(proposal_id: Index, payload: &ProposalPayload) -> bool {
    let validated = validate_payload(proposal_id, payload);
    let args_raw = match validated {
        true => {
            let (voting_end_time, threshold) = set_threshold(payload);

            encode_args((proposal_id, Some(voting_end_time), Some(threshold), true)).unwrap()
        }
        false => encode_args((
            proposal_id,
            None::<TimeNs>,
            None::<ProposalPassingThreshold>,
            false,
        ))
        .unwrap(),
    };

    let _ = ic_cdk::api::call::call_raw128(get_governance(), "validate", &args_raw, 0).await;
    validated
}

/// Modify this function for a more sophisticated payload validation strategy.
/// Proposal type may be determined by inspecting the payload.
/// This example uses a simple canister id whitelist.
pub fn validate_payload(proposal: Index, payload: &ProposalPayload) -> bool {
    // Cannot depend on itself or future proposals
    if payload.depends_on.iter().max().cloned() >= Some(proposal) {
        return false;
    }
    // Check if all canisters are in the whitelist
    if payload
        .messages
        .iter()
        .any(|m| !CALL_TARGET_WHITELIST.with(|c| c.borrow().contains_key(&m.canister_id.into())))
    {
        return false;
    }
    // Verify that no pre- nor post-validation is used
    if payload
        .messages
        .iter()
        .any(|m| m.pre_validate.is_some() || m.post_validate.is_some())
    {
        return false;
    }
    true
}

/// Modify this function to set the voting end time and the passing threshold for a proposal according to the payload.
/// Proposal type may be determined by inspecting the payload.
pub fn set_threshold(_payload: &ProposalPayload) -> (TimeNs, ProposalPassingThreshold) {
    (
        (86400 * 5) * 1_000_000_000 + ic_cdk::api::time(),
        ProposalPassingThreshold {
            quorum: Percentage::<PercentagePrecision>::from_percent(20),
            passing_threshold: Percentage::<PercentagePrecision>::from_percent(20),
        },
    )
}

#[update]
pub fn set_governance(canister_id: Principal) {
    require_caller_has_role(UserRole::Admin);
    CONFIG.with(|c| {
        let mut c = c.borrow_mut();
        let mut config = c.get().0.clone().unwrap();
        config.governance_canister = canister_id;
        c.set(Cbor(Some(config))).expect("config update failed");
    });
}

#[update]
pub fn set_config(config: Config) {
    require_caller_has_role(UserRole::Admin);
    CONFIG.with(|c| {
        let mut c = c.borrow_mut();
        c.set(Cbor(Some(config))).expect("config update failed");
    });
}

#[update]
pub fn add_call_targets_to_whitelist(canister_ids: Vec<Principal>) {
    require_caller_has_role(UserRole::Admin);
    CALL_TARGET_WHITELIST.with(|c| {
        let mut c = c.borrow_mut();
        for canister_id in canister_ids {
            c.insert(canister_id.into(), ());
        }
    });
}

#[query]
pub fn get_name() -> String {
    CONFIG.with(|c| c.borrow().get().0.clone().unwrap().name)
}

#[query]
pub fn get_description() -> String {
    CONFIG.with(|c| c.borrow().get().0.clone().unwrap().description)
}

#[query]
pub fn is_initialized() -> bool {
    CONFIG.with(|c| c.borrow().get().0.clone().unwrap().initialized)
}

#[query]
pub fn get_governance() -> Principal {
    CONFIG.with(|c| {
        let config = c.borrow().get().0.clone().unwrap();
        config.governance_canister
    })
}

#[query]
pub fn get_call_target_whitelist() -> Vec<Principal> {
    CALL_TARGET_WHITELIST.with(|c| c.borrow().iter().map(|(x, _)| x.into()).collect())
}

#[cfg(any(target_arch = "wasm32", test))]
ic_cdk::export_candid!();

fn main() {}
