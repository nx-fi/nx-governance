//! This is a simple multisig voting canister to be used with NX Governance.
//! In order to use this canister, use a validator (such as simple-validator)
//! that sets passing threshold to any arbitrary value.

mod access;
mod lifecycle;
mod memory;
mod types;

use crate::access::*;
use crate::memory::*;
use crate::types::*;

use candid::{decode_one, encode_args, encode_one, Principal};
use ic_cdk_macros::{query, update};

/// Notification called by the governance canister to notify the multisig canister that a proposal is ready for voting.
#[update]
pub async fn notify_multisig(proposal: Index, voting_end_time: TimeNs) {
    assert_eq!(ic_cdk::api::caller(), get_governance());

    PROPOSAL_VOTES.with(|v| {
        let mut v = v.borrow_mut();
        v.insert(
            proposal,
            Cbor(ProposalState {
                expiration: voting_end_time,
                state: State::Open,
                vote_record: VoteRecord::default(),
            }),
        );
    });
}

#[update]
pub async fn sync_with_governance() -> Result<(), ReturnError> {
    let config = memory::get_config()?;
    let res = ic_cdk::api::call::call_raw128(
        config.governance_canister,
        "get_all_open_proposal_ids_with_expiration",
        &encode_one(()).unwrap(),
        0,
    )
    .await
    .map_err(|_| ReturnError::InterCanisterCallError)?;
    let result: Vec<(Index, TimeNs)> = decode_one(&res).unwrap();

    // Set PropsalState for each non-existing proposal
    for (proposal, expiration) in result {
        add_proposal_state(
            proposal,
            ProposalState {
                expiration,
                state: State::Open,
                vote_record: VoteRecord::default(),
            },
        );
    }

    Ok(())
}

/// Each signer calls this function to vote on a proposal.
#[update]
pub fn vote_proposal(proposal: Index, vote: Vote) -> Result<(), ReturnError> {
    require_caller_is_signer();
    let caller = ic_cdk::api::caller();

    PROPOSAL_VOTES.with(|v| {
        let mut v = v.borrow_mut();
        let mut proposal_state = v.get(&proposal).unwrap().0.clone();
        assert!(
            !proposal_state.vote_record.yes_votes.contains(&caller)
                && !proposal_state.vote_record.no_votes.contains(&caller)
                && !proposal_state.vote_record.abstain_votes.contains(&caller),
            "caller has already voted"
        );
        if ic_cdk::api::time() > proposal_state.expiration - get_vote_buffer_time() {
            return Err(ReturnError::Expired);
        }
        match vote {
            Vote::Yes => {
                proposal_state.vote_record.yes_votes.push(caller);
            }
            Vote::No => {
                proposal_state.vote_record.no_votes.push(caller);
            }
            Vote::Abstain => {
                proposal_state.vote_record.abstain_votes.push(caller);
            }
        }
        v.insert(proposal, Cbor(proposal_state));
        Ok(())
    })
}

/// Signer can call this function to submit vote result back to governance.
/// Can only be called when threshold is reached.
/// The submitted result is simply a yes/no voting power of 1 with a total voting power of 1.
#[update]
pub async fn submit_vote_result(proposal: Index) -> Result<(), ReturnError> {
    require_caller_is_signer();

    let (pass_threshold, _) = get_m_of_n();

    let vote_passed = PROPOSAL_VOTES.with(|v| {
        let mut v = v.borrow_mut();
        let mut proposal_state = v.get(&proposal).unwrap().0.clone();
        // If already passed then return true to allow re-sync in case of error.
        if proposal_state.state == State::Passed {
            return Ok(true);
        }
        // Check if voting has expired.
        if ic_cdk::api::time() > proposal_state.expiration {
            proposal_state.state = State::Failed;
            v.insert(proposal, Cbor(proposal_state));
            return Err(ReturnError::Expired);
        }
        // Check if threshold is reached.
        let yes_votes = proposal_state.vote_record.yes_votes.len() as u64;
        let passed = yes_votes >= pass_threshold;
        if passed {
            proposal_state.state = State::Passed;
        }
        // Update state.
        v.insert(proposal, Cbor(proposal_state));
        Ok(passed)
    })?;
    // If a vote has not passed, it simply won't be submitted.
    if vote_passed {
        submit_result(proposal, 1, 0, 0, 1).await
    } else {
        Err(ReturnError::GenericError)
    }
}

async fn submit_result(
    proposal: Index,
    yes_voting_power: VotingPower,
    no_voting_power: VotingPower,
    abstain_voting_power: VotingPower,
    total_voting_power: VotingPower,
) -> Result<(), ReturnError> {
    let args_raw = encode_args((
        proposal,
        yes_voting_power,
        no_voting_power,
        abstain_voting_power,
        total_voting_power,
    ));
    let res = ic_cdk::api::call::call_raw128(
        get_governance(),
        "update_vote_result_and_total_voting_power",
        &args_raw.unwrap(),
        0,
    )
    .await
    .map_err(|_| ReturnError::InterCanisterCallError)?;
    let result: Result<(), ReturnError> = decode_one(&res).unwrap();
    result
}

pub fn require_caller_is_signer() {
    SIGNERS.with(|s| {
        if !s.borrow().contains_key(&ic_cdk::api::caller().into()) {
            ic_cdk::trap("Caller is not a signer");
        }
    });
}

/// Update the m-of-n configuration of the multisig.
/// It could be dangerous to update the m-of-n configuration when there are open proposals.
/// # Panics
/// Panics if `votes_required` is greater than `total_votes`.
/// Panics if `total_votes` is not equal to the number of signers.
/// Panics if any memory operation fails.
#[update]
pub fn update_m_of_n(votes_required: u64, total_votes: u64, signers: Vec<Principal>) {
    require_caller_has_role(UserRole::Admin);
    assert!(votes_required <= total_votes);
    assert_eq!(total_votes, signers.len() as u64);

    CONFIG.with(|c| {
        let mut c = c.borrow_mut();
        let mut config = c.get().0.clone().unwrap();
        config.votes_required = votes_required;
        config.total_votes = total_votes;

        c.set(Cbor(Some(config))).expect("config update failed");
    });

    SIGNERS.with(|s| {
        let mut s = s.borrow_mut();
        let keys: Vec<StablePrincipal> = s.iter().map(|(k, _)| k).collect();
        for key in keys {
            s.remove(&key);
        }
        for signer in signers {
            s.insert(signer.into(), ());
        }
        assert_eq!(s.len(), total_votes);
    });
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
pub fn set_name_description(name: String, description: String) {
    require_caller_has_role(UserRole::Admin);
    CONFIG.with(|c| {
        let mut c = c.borrow_mut();
        let mut config = c.get().0.clone().unwrap();
        config.name = name;
        config.description = description;
        c.set(Cbor(Some(config))).expect("config update failed");
    });
}

#[query]
pub fn get_num_proposals() -> u64 {
    PROPOSAL_VOTES.with(|v| v.borrow().len())
}

#[query]
pub fn get_open_proposals() -> Vec<Index> {
    PROPOSAL_VOTES.with(|v| {
        let v = v.borrow();
        v.iter()
            .filter_map(|(k, _)| {
                if v.get(&k).unwrap().0.state == State::Open {
                    Some(k)
                } else {
                    None
                }
            })
            .collect()
    })
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
pub fn get_m_of_n() -> (u64, u64) {
    CONFIG.with(|c| {
        let config = c.borrow().get().0.clone().unwrap();
        (config.votes_required, config.total_votes)
    })
}

#[query]
pub fn get_vote_buffer_time() -> u64 {
    CONFIG.with(|c| {
        let config = c.borrow().get().0.clone().unwrap();
        config.vote_buffer_time
    })
}

#[cfg(any(target_arch = "wasm32", test))]
ic_cdk::export_candid!();

fn main() {}
