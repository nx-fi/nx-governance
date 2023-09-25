#![doc = include_str!("../../../README.md")]

// TODO: governance recovery mechanism

mod access;
mod execution;
pub mod http;
mod lifecycle;
mod memory;
mod metrics;
mod proposal;
mod storage;
//mod timer;
mod types;
mod validate;

use crate::access::*;
use crate::execution::*;
#[allow(unused_imports)]
use crate::http::*;
use crate::memory::*;
use crate::proposal::*;
use crate::storage::*;
use crate::types::*;
use crate::validate::Validate;

use candid::{decode_one, encode_one, Principal};
use ic_cdk::api::{
    call::CallResult,
    management_canister::main::{
        canister_info, canister_status, CanisterIdRecord, CanisterInfoRequest,
        CanisterInfoResponse, CanisterStatusResponse, CanisterStatusType,
    },
};
use ic_cdk_macros::{query, update};

use num_traits::cast::ToPrimitive;

// ==== Proposal functions ====
/// Submit a proposal. Returns the the proposal ID.
/// # Panics
/// Panics if the proposal is invalid.
#[update]
pub fn submit(
    metadata: ProposalMetadata,
    payload: ProposalPayload,
    activates: Schedule,
    expires: Schedule,
    auto_execute: bool,
) -> Result<Index, ReturnError> {
    require_caller_has_role(UserRole::Proposer);
    assert!(metadata.is_valid() && payload.is_valid() && expires.is_in_future());

    let caller = ic_cdk::api::caller();
    let metadata_id = add_proposal_metadata(&metadata)?;
    let payload_id = add_proposal_payload(&payload)?;

    let proposal = Proposal::from_submit(
        metadata_id,
        payload_id,
        auto_execute,
        activates,
        expires,
        &caller,
    );
    let proposal_id = add_proposal(&proposal)?;
    assert!(payload.max_dependency_index() < Some(proposal_id));

    if get_config()?.validator_hook.is_some() {
        push_timer_task(proposal_id)?;
    }

    Ok(proposal_id)
}

/// Validate a proposal.
/// This function is called by the validator canister.
#[update]
pub fn validate(
    proposal_id: Index,
    voting_end_time: Option<TimeNs>,
    passing_threshold: Option<ProposalPassingThreshold>,
    validated: bool,
) -> Result<(), ReturnError> {
    require_caller_has_role(UserRole::Validator);

    let config = get_config()?;
    // must set voting_end_time and passing_threshold after this stage if validated
    if validated
        && (!passing_threshold
            .clone()
            .is_some_and(|x| x.is_valid() && x.all_fields_gte(&config.min_passing_threshold))
            || !voting_end_time.is_some_and(|x| x > ic_cdk::api::time() + config.min_voting_period))
    {
        return Err(ReturnError::InputError);
    }

    let mut proposal = get_proposal_by_id(proposal_id)?;

    if proposal.state != ProposalState::Submitted {
        return Err(ReturnError::IncorrectProposalState);
    }

    let _ = match validated {
        true => proposal
            .state_transition(ProposalState::Open)
            .map_err(|_| ReturnError::StateTransitionError)?,
        false => proposal
            .state_transition(ProposalState::ValidationFailed)
            .map_err(|_| ReturnError::StateTransitionError)?,
    };
    proposal.validated = Some(validated);
    proposal.voting_end_time = voting_end_time;
    proposal.passing_threshold = passing_threshold;

    set_proposal_by_id(proposal_id, &proposal);

    if get_config()?.vote_manager_hook.is_some() {
        push_timer_task(proposal_id)?;
    }

    Ok(())
}

/// Update vote results.
/// This function is called by the vote manager canister.
#[update]
pub fn update_vote_result(
    proposal_id: Index,
    yes_voting_power: VotingPower,
    no_voting_power: VotingPower,
    abstain_voting_power: VotingPower,
) -> Result<(), ReturnError> {
    require_caller_has_role(UserRole::VoteManager);
    let mut proposal = get_proposal_by_id(proposal_id)?;
    if !proposal.is_voteable() {
        return Err(ReturnError::Expired);
    }
    proposal.votes_yes += yes_voting_power;
    proposal.votes_no += no_voting_power;
    proposal.votes_abstain += abstain_voting_power;
    if proposal.votes_yes < 0
        || proposal.votes_no < 0
        || proposal.votes_abstain < 0
        || proposal.votes_yes + proposal.votes_no + proposal.votes_abstain
            >= proposal.total_voting_power
    {
        return Err(ReturnError::ArithmeticError);
    }
    let config = get_config()?;
    if config.voting_may_end_early || proposal.voting_end_time.unwrap() < ic_cdk::api::time() {
        proposal
            .try_finalize_vote_result()
            .map_err(|_| ReturnError::StateTransitionError)?;
    }

    set_proposal_by_id(proposal_id, &proposal);
    Ok(())
}

/// Update the total voting power for a proposal.
///
/// `total_voting_power` is the absolute value of total voting power of all the voters.
/// This function is called by the vote manager canister.
/// Depending on the voting mechanism, this function is called either only once at the beginning,
/// or updated on an ongoing basis. If it's updated on an ongoing basis,
/// then the order and timing of calling this function and `update_vote_result` is important,
/// depending on whether voting power increases or decreases.
/// If the update order or timing is incorrect, the proposal may be incorrectly accepted or rejected.
#[update]
pub fn update_total_voting_power(
    proposal_id: Index,
    total_voting_power: VotingPower,
) -> Result<(), ReturnError> {
    require_caller_has_role(UserRole::VoteManager);
    let mut proposal = get_proposal_by_id(proposal_id)?;
    if !proposal.is_voteable() {
        return Err(ReturnError::Expired);
    }
    if total_voting_power < 0 {
        return Err(ReturnError::ArithmeticError);
    }
    proposal.total_voting_power = total_voting_power;
    let config = get_config()?;
    if config.voting_may_end_early || proposal.voting_end_time.unwrap() < ic_cdk::api::time() {
        proposal
            .try_finalize_vote_result()
            .map_err(|_| ReturnError::StateTransitionError)?;
    }
    set_proposal_by_id(proposal_id, &proposal);
    Ok(())
}

/// Update results and voting power at the same time.
///
/// This function is called by the vote manager canister.
/// `total_voting_power` is the absolute value of total voting power of all the voters.
/// `yes_voting_power`, `no_voting_power`, `abstain_voting_power` are incremental changes of voting power.
#[update]
pub fn update_vote_result_and_total_voting_power(
    proposal_id: Index,
    yes_voting_power: VotingPower,
    no_voting_power: VotingPower,
    abstain_voting_power: VotingPower,
    total_voting_power: VotingPower,
) -> Result<(), ReturnError> {
    require_caller_has_role(UserRole::VoteManager);
    let mut proposal = get_proposal_by_id(proposal_id)?;
    if !proposal.is_voteable() {
        return Err(ReturnError::Expired);
    }
    if total_voting_power < 0 {
        return Err(ReturnError::ArithmeticError);
    }
    proposal.votes_yes += yes_voting_power;
    proposal.votes_no += no_voting_power;
    proposal.votes_abstain += abstain_voting_power;
    proposal.total_voting_power = total_voting_power;
    if proposal.votes_yes < 0 || proposal.votes_no < 0 || proposal.votes_abstain < 0 {
        return Err(ReturnError::ArithmeticError);
    }
    let config = get_config()?;
    if config.voting_may_end_early || proposal.voting_end_time.unwrap() < ic_cdk::api::time() {
        proposal
            .try_finalize_vote_result()
            .map_err(|_| ReturnError::StateTransitionError)?;
    }
    set_proposal_by_id(proposal_id, &proposal);
    Ok(())
}

/// Finalize a proposal.
/// This function is called by the vote manager canister.
#[update]
pub fn finalize_vote_result(proposal_id: Index) -> Result<(), ReturnError> {
    require_caller_has_role(UserRole::VoteManager);
    let mut proposal = get_proposal_by_id(proposal_id)?;
    let config = get_config()?;
    if config.voting_may_end_early || proposal.voting_end_time.unwrap() < ic_cdk::api::time() {
        proposal
            .try_finalize_vote_result()
            .map_err(|_| ReturnError::StateTransitionError)?;
    }
    set_proposal_by_id(proposal_id, &proposal);
    Ok(())
}

/// Revoke a proposal.
/// Returns the revoke index
/// This function is called by the revoker.
#[update]
pub fn revoke(proposal_id: Index, reason: String) -> Result<(), ReturnError> {
    require_caller_has_role(UserRole::Revoker);
    let mut proposal = get_proposal_by_id(proposal_id)?;
    if proposal.state != ProposalState::Open {
        return Err(ReturnError::IncorrectProposalState);
    }
    proposal
        .state_transition(ProposalState::Revoked)
        .map_err(|_| ReturnError::StateTransitionError)?;
    set_proposal_by_id(proposal_id, &proposal);
    add_proposal_revoke(&ProposalRevoke {
        proposal_id,
        reason,
        revoked_at: ic_cdk::api::time(),
    })?;
    Ok(())
}

/// Execute a proposal.
/// This function is called by the executor.
#[update]
pub async fn execute(proposal_id: Index) -> Result<(), ReturnError> {
    require_caller_has_role(UserRole::Executor);

    let mut proposal = get_proposal_by_id(proposal_id)?;
    if proposal.state == ProposalState::Open {
        proposal
            .try_finalize_vote_result()
            .map_err(|_| ReturnError::StateTransitionError)?;
    }
    if !proposal.is_executable() {
        return Err(ReturnError::IncorrectProposalState);
    }
    let proposal_payload = get_proposal_payload_by_id(proposal.payload_id)?;

    validate_execution_dependency(proposal_payload.depends_on)?;

    // Execute
    let mut err_flag = false;
    for (i, message) in proposal_payload.messages.iter().enumerate() {
        proposal
            .state_transition(ProposalState::Executing(ExecutionStep::new(i as u8)))
            .map_err(|_| ReturnError::StateTransitionError)?;
        set_proposal_by_id(proposal_id, &proposal);

        let res = execute_message(message, proposal_id).await;

        if res.is_err() {
            // this pattern matching is guaranteed to succeed
            if let ProposalState::Executing(ref exec_step) = proposal.state {
                let _ = proposal
                    .state_transition(ProposalState::Failed(exec_step.clone()))
                    .map_err(|_| ReturnError::StateTransitionError)?;
                set_proposal_by_id(proposal_id, &proposal);
            }
            err_flag = true;
            break;
        }
    }
    if !err_flag {
        let _ = proposal
            .state_transition(ProposalState::Succeeded)
            .map_err(|_| ReturnError::StateTransitionError)?;
        set_proposal_by_id(proposal_id, &proposal);
    }
    Ok(())
}

/// Force execute a proposal.
/// This function is called by the force executor.
#[update]
pub async fn force_execute(proposal_id: Index) -> Result<(), ReturnError> {
    require_caller_has_role(UserRole::ForceExecutor);

    let mut proposal = get_proposal_by_id(proposal_id)?;
    if !proposal.is_force_executable() {
        return Err(ReturnError::IncorrectProposalState);
    }
    let proposal_payload = get_proposal_payload_by_id(proposal.payload_id)?;

    validate_execution_dependency(proposal_payload.depends_on)?;

    // Execute
    let mut err_flag = false;
    for (i, message) in proposal_payload.messages.iter().enumerate() {
        proposal
            .state_transition(ProposalState::ForceExecuting(ExecutionStep::new(i as u8)))
            .map_err(|_| ReturnError::StateTransitionError)?;
        set_proposal_by_id(proposal_id, &proposal);

        let res = execute_message(message, proposal_id).await;

        if res.is_err() {
            // this pattern matching is guaranteed to succeed
            if let ProposalState::ForceExecuting(ref exec_step) = proposal.state {
                let _ = proposal
                    .state_transition(ProposalState::ForceExecutionFailed(exec_step.clone()))
                    .map_err(|_| ReturnError::StateTransitionError)?;
                set_proposal_by_id(proposal_id, &proposal);
            }
            err_flag = true;
            break;
        }
    }
    if !err_flag {
        let _ = proposal
            .state_transition(ProposalState::ForceExecutionSucceeded)
            .map_err(|_| ReturnError::StateTransitionError)?;
        set_proposal_by_id(proposal_id, &proposal);
    }
    Ok(())
}

fn validate_execution_dependency(deps: Vec<Index>) -> Result<(), ReturnError> {
    for &dep in deps.iter() {
        let dependent_proposal = get_proposal_by_id(dep)?;
        match dependent_proposal.state {
            ProposalState::Succeeded | ProposalState::ForceExecutionSucceeded => {}
            ProposalState::Failed(_)
            | ProposalState::ForceExecutionFailed(_)
            | ProposalState::ValidationFailed
            | ProposalState::Expired
            | ProposalState::Rejected
            | ProposalState::Revoked
            | ProposalState::QuorumNotMet => {
                return Err(ReturnError::DependentProposalNotSucceeded)
            }
            // caller can retry execute this proposal
            ProposalState::Submitted
            | ProposalState::Open
            | ProposalState::Accepted
            | ProposalState::Executing(_)
            | ProposalState::ForceExecuting(_) => {
                return Err(ReturnError::DependentProposalNotReady)
            }
        }
    }
    Ok(())
}

/// A single `message` is executed, modifying state of the proposal.
async fn execute_message(message: &CanisterMessage, proposal_id: Index) -> Result<(), ReturnError> {
    let mut proposal = get_proposal_by_id(proposal_id)?;

    // Pre validation
    if message.pre_validate.is_some() {
        let target = message.pre_validate.as_ref().unwrap();
        let pre_validate_res = ic_cdk::api::call::call_raw128(
            target.canister_id,
            &target.method,
            target.payload.clone(),
            target.payment,
        )
        .await;
        match pre_validate_res {
            Ok(res) => {
                let res = decode_one(&res).unwrap();
                match res {
                    true => {
                        let _ = proposal
                            .execution_state_transition(ExecutionStepState::Executing)
                            .map_err(|_| ReturnError::StateTransitionError)?;
                    }
                    false => {
                        let _ = proposal
                            .execution_state_transition(ExecutionStepState::PreValidateFailed)
                            .map_err(|_| ReturnError::StateTransitionError)?;
                        set_proposal_by_id(proposal_id, &proposal);
                        return Err(ReturnError::PreValidateFailed);
                    }
                }
            }
            Err(_) => {
                let _ = proposal
                    .execution_state_transition(ExecutionStepState::PreValidateCallError)
                    .map_err(|_| ReturnError::StateTransitionError)?;
                set_proposal_by_id(proposal_id, &proposal);
                return Err(ReturnError::InterCanisterCallError);
            }
        };
    }

    // Execution
    let exec_res = ic_cdk::api::call::call_raw128(
        message.canister_id,
        &message.method,
        message.message.clone(),
        message.payment,
    )
    .await
    .map_err(|(code, message)| (code as i32, message));
    add_execution_step_result(proposal_id, ExecResult(exec_res.clone()));
    match exec_res {
        Ok(_) => {
            let _ = proposal
                .execution_state_transition(ExecutionStepState::PostValidating)
                .map_err(|_| ReturnError::StateTransitionError)?;
            set_proposal_by_id(proposal_id, &proposal);
        }
        Err(_) => {
            let _ = proposal
                .execution_state_transition(ExecutionStepState::ExecutionCallError)
                .map_err(|_| ReturnError::StateTransitionError)?;
            set_proposal_by_id(proposal_id, &proposal);
            return Err(ReturnError::InterCanisterCallError);
        }
    };

    // Post validation
    if message.post_validate.is_some() {
        let target = message.post_validate.as_ref().unwrap();
        #[allow(clippy::unwrap_used)]
        let payload = PostValidatePayload {
            canister_id: message.canister_id,
            method: message.method.clone(),
            message: message.message.clone(),
            response: exec_res.unwrap(),
        };
        let post_validate_res = ic_cdk::api::call::call_raw128(
            target.canister_id,
            &target.method,
            encode_one(payload).unwrap(),
            target.payment,
        )
        .await;
        match post_validate_res {
            Ok(res) => {
                let res = decode_one(&res).unwrap();
                match res {
                    true => {
                        let _ = proposal
                            .execution_state_transition(ExecutionStepState::Succeeded)
                            .map_err(|_| ReturnError::StateTransitionError)?;
                    }
                    false => {
                        let _ = proposal
                            .execution_state_transition(ExecutionStepState::PostValidateFailed)
                            .map_err(|_| ReturnError::StateTransitionError)?;
                        set_proposal_by_id(proposal_id, &proposal);
                        return Err(ReturnError::PostValidateFailed);
                    }
                }
            }
            Err(_) => {
                let _ = proposal
                    .execution_state_transition(ExecutionStepState::PostValidateCallError)
                    .map_err(|_| ReturnError::StateTransitionError)?;
                set_proposal_by_id(proposal_id, &proposal);
                return Err(ReturnError::InterCanisterCallError);
            }
        };
    } else {
        let _ = proposal
            .execution_state_transition(ExecutionStepState::Succeeded)
            .map_err(|_| ReturnError::StateTransitionError)?;
    }
    set_proposal_by_id(proposal_id, &proposal);
    Ok(())
}

// ==== Settings ====

#[update]
pub fn update_config(config: Config) -> Result<(), ReturnError> {
    require_caller_has_role(UserRole::Admin);
    CONFIG
        .with(|c| c.borrow_mut().set(Cbor(Some(config))))
        .map_err(|_| ReturnError::MemoryError)?;
    Ok(())
}

// TODO: add settings related functions

// ==== Getters ====

#[query]
pub fn get_all_open_proposal_ids_with_expiration() -> Vec<(Index, TimeNs)> {
    PROPOSALS.with(|p| {
        p.borrow()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if p.state == ProposalState::Open {
                    Some((i as Index, p.voting_end_time.unwrap()))
                } else {
                    None
                }
            })
            .collect()
    })
}

#[query]
pub fn get_all_submitted_proposal_ids() -> Vec<Index> {
    PROPOSALS.with(|p| {
        p.borrow()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if p.state == ProposalState::Submitted {
                    Some(i as Index)
                } else {
                    None
                }
            })
            .collect()
    })
}

#[query]
pub fn get_proposal_states(skip: Index, take: Index) -> Vec<ProposalState> {
    PROPOSALS.with(|p| {
        p.borrow()
            .iter()
            .skip(skip as usize)
            .take(take as usize)
            .map(|p| p.state.clone())
            .collect()
    })
}

#[query]
pub fn get_next_proposal_id() -> Index {
    PROPOSALS.with(|p| p.borrow().len() as Index)
}

#[query]
pub fn get_proposal(proposal_id: Index) -> Option<Proposal> {
    PROPOSALS.with(|p| p.borrow().get(proposal_id))
}

#[query]
pub fn get_proposal_metadata(proposal_id: Index) -> Option<ProposalMetadata> {
    PROPOSAL_METADATA.with(|p| p.borrow().get(proposal_id))
}

#[query]
pub fn get_proposal_payload(proposal_id: Index) -> Option<ProposalPayload> {
    PROPOSAL_PAYLOAD.with(|p| p.borrow().get(proposal_id))
}

#[query]
pub fn get_proposal_exec(proposal_id: Index) -> Option<ProposalExec> {
    PROPOSAL_EXEC.with(|p| p.borrow().get(&proposal_id))
}

#[query]
pub fn get_proposal_revoke(proposal_id: Index) -> Option<ProposalRevoke> {
    PROPOSAL_REVOKE.with(|p| p.borrow().get(proposal_id))
}

// ==== Target canister getters ====
// Controller-only statuses of canisters under management are exposed without access control.
// OPT: add text interface
// OPT: add bool validation interface with Greater(Value), GreaterOrEqual(), Equal(Value), LessOrEqual(), Less(Value) constraints

#[query]
pub async fn get_controllers_of(canister_id: Principal) -> Vec<Principal> {
    get_info_of(canister_id, None).await.controllers
}

#[query]
pub async fn get_module_hash_of(canister_id: Principal) -> Option<Vec<u8>> {
    get_info_of(canister_id, None).await.module_hash
}

#[rustfmt::skip]
#[query]
pub async fn get_cycle_balance_of(canister_id: Principal) -> u128 {
    get_status_of(canister_id).await.cycles.0.to_u128().unwrap()
}

#[rustfmt::skip]
#[query]
pub async fn get_freezing_threshold_of(canister_id: Principal) -> u128 {
    get_status_of(canister_id).await.settings.freezing_threshold.0.to_u128().unwrap()
}

#[query]
pub async fn get_stopping_status_of(canister_id: Principal) -> CanisterStatusType {
    get_status_of(canister_id).await.status
}

#[query]
pub async fn get_status_of(canister_id: Principal) -> CanisterStatusResponse {
    let call_result: CallResult<(CanisterStatusResponse,)> =
        canister_status(CanisterIdRecord { canister_id }).await;
    let (status,) = call_result.unwrap();
    status
}

#[query]
pub async fn get_info_of(
    canister_id: Principal,
    num_requested_changes: Option<u64>,
) -> CanisterInfoResponse {
    let call_result: CallResult<(CanisterInfoResponse,)> = canister_info(CanisterInfoRequest {
        canister_id,
        num_requested_changes,
    })
    .await;
    let (info,) = call_result.unwrap();
    info
}

#[cfg(any(target_arch = "wasm32", test))]
ic_cdk::export_candid!();

fn main() {}
