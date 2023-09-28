use crate::types::*;
use crate::validate::Validate;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// Proposal ID is equal to its index.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Proposal {
    // ---- Provided at creation time ----
    /// Index to the metadata of the proposal.
    pub metadata_id: Index,
    /// Index to the payload of the proposal.
    pub payload_id: Index,
    /// Automatically execute proposals using the timer. Canister must add itself as executor role.
    pub auto_execute: bool,
    /// Activation time, after which an proposal can be executed.
    ///
    /// If activation time is in the past it can be executed immediately.
    /// Force execute cannot bypass this condition.
    /// Conversion to absolute time happens when the proposal voting is finalized.
    /// If created with relative time, the proposal cannot be force executed.
    pub activates: Schedule,
    /// Expiration time, after which an proposal can no longer be executed.
    ///
    /// Conversion to absolute time happens when the proposal voting is finalized.
    /// If created with relative time, the proposal cannot be force executed.
    pub expires: Schedule,

    // ---- Generated at creation time ----
    /// Time when the proposal was created.
    pub created_at: TimeNs,
    /// The proposer of the proposal.
    pub proposer: Principal,

    // ---- Set by validator ----
    /// Validation status. OPT: duplicate of `state`.
    /// After a proposal is created, it can trigger an external configuration/validation process.
    pub validated: Option<bool>,
    /// Time when the voting period would end. Set when the proposal is created according to type.
    pub voting_end_time: Option<TimeNs>,
    /// Passing threshold of the proposal. Set when the proposal is created according to type.
    pub passing_threshold: Option<ProposalPassingThreshold>,

    // ---- Set by state machine control flow ----
    /// The state of the proposal.
    pub state: ProposalState,

    // ---- Set by vote manager ----
    /// The voting power of yes votes.
    pub votes_yes: VotingPower,
    /// The voting power of no votes.
    pub votes_no: VotingPower,
    /// The voting power of abstain votes.
    pub votes_abstain: VotingPower,
    /// Total voting power valid for the current proposal.
    pub total_voting_power: VotingPower,
}

#[allow(clippy::enum_variant_names)]
#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ProposalError {
    GenericError,
    StateTransitionError,
    ArithmeticError,
}

impl Proposal {
    pub fn from_submit(
        metadata_id: Index,
        payload_id: Index,
        auto_execute: bool,
        activates: Schedule,
        expires: Schedule,
        proposer: &Principal,
    ) -> Self {
        Self {
            metadata_id,
            payload_id,
            auto_execute,
            activates,
            expires,
            created_at: ic_cdk::api::time(),
            proposer: proposer.to_owned(),
            validated: None,
            voting_end_time: None,
            passing_threshold: None,
            state: ProposalState::Submitted,
            votes_yes: 0,
            votes_no: 0,
            votes_abstain: 0,
            total_voting_power: 0,
        }
    }

    pub fn is_voteable(&self) -> bool {
        self.state == ProposalState::Open && self.voting_end_time > Some(ic_cdk::api::time())
    }

    pub fn is_expired(&self) -> bool {
        Some(ic_cdk::api::time()) > self.voting_end_time
    }

    pub fn is_executable(&self) -> bool {
        self.state == ProposalState::Accepted
            && self.activates.is_absolute()
            && self.activates.to_timestamp() <= Schedule::At(ic_cdk::api::time()).to_timestamp()
            && self.expires.is_absolute()
            && self.expires.to_timestamp() > Schedule::At(ic_cdk::api::time()).to_timestamp()
    }

    pub fn is_force_executable(&self) -> bool {
        self.state == ProposalState::Open
            && self.activates.is_absolute()
            && self.activates.to_timestamp() <= Schedule::At(ic_cdk::api::time()).to_timestamp()
            && self.expires.is_absolute()
            && self.expires.to_timestamp() > Schedule::At(ic_cdk::api::time()).to_timestamp()
    }

    /// Finalize the activation time.
    pub fn finalize_activation(&mut self) {
        self.activates.convert_to_absolute();
    }

    /// Finalize the expiration time.
    pub fn finalize_expiration(&mut self) {
        self.expires.convert_to_absolute();
    }

    /// Ensures proper state transitions of the state machine.
    /// Returns previous state if transition successful.
    pub fn state_transition(
        &mut self,
        next_state: ProposalState,
    ) -> Result<ProposalState, ProposalError> {
        match self.state {
            ProposalState::Submitted => match next_state {
                ProposalState::Open | ProposalState::ValidationFailed => {
                    self.state = next_state;
                    Ok(ProposalState::Submitted)
                }
                _ => Err(ProposalError::StateTransitionError),
            },
            ProposalState::Open => match next_state {
                ProposalState::Accepted
                | ProposalState::Rejected
                | ProposalState::Revoked
                | ProposalState::QuorumNotMet
                | ProposalState::ForceExecuting(_) => {
                    self.state = next_state;
                    Ok(ProposalState::Open)
                }
                _ => Err(ProposalError::StateTransitionError),
            },
            ProposalState::Accepted => match next_state {
                ProposalState::Executing(_) => {
                    self.state = next_state;
                    Ok(ProposalState::Accepted)
                }
                _ => Err(ProposalError::StateTransitionError),
            },
            ProposalState::Executing(_) => match next_state {
                ProposalState::Executing(_)
                | ProposalState::Succeeded
                | ProposalState::Failed(_)
                | ProposalState::Expired => {
                    let prev_state = self.state.clone();
                    self.state = next_state;
                    Ok(prev_state)
                }
                _ => Err(ProposalError::StateTransitionError),
            },
            ProposalState::ForceExecuting(_) => match next_state {
                ProposalState::ForceExecuting(_)
                | ProposalState::ForceExecutionSucceeded
                | ProposalState::ForceExecutionFailed(_) => {
                    let prev_state = self.state.clone();
                    self.state = next_state;
                    Ok(prev_state)
                }
                _ => Err(ProposalError::StateTransitionError),
            },
            _ => Err(ProposalError::StateTransitionError),
        }
    }

    pub fn execution_state_transition(
        &mut self,
        exec_step_state: ExecutionStepState,
    ) -> Result<ProposalState, ProposalError> {
        let prev_state = self.state.clone();
        match self.state.clone() {
            ProposalState::Executing(mut s) => {
                let _ = s
                    .state_transition(exec_step_state)
                    .map_err(|_| ProposalError::StateTransitionError)?;
                self.state = ProposalState::Executing(s);
                Ok(prev_state)
            }
            ProposalState::ForceExecuting(mut s) => {
                let _ = s
                    .state_transition(exec_step_state)
                    .map_err(|_| ProposalError::StateTransitionError)?;
                self.state = ProposalState::ForceExecuting(s);
                Ok(prev_state)
            }
            _ => Err(ProposalError::StateTransitionError),
        }
    }

    /// Votes of `yes`, `no`, and `abstain` all count towards the quorum.
    pub fn current_participation_rate(&self) -> Percentage<PercentagePrecision> {
        let total_votes = self.votes_yes as f64 + self.votes_no as f64 + self.votes_abstain as f64;
        (total_votes / self.total_voting_power as f64).into()
    }

    /// Only votes of `yes` and `no` count towards the pass rate.
    pub fn current_yes_rate(&self) -> Percentage<PercentagePrecision> {
        let effective_votes = self.votes_yes as f64 + self.votes_no as f64;
        (self.votes_yes as f64 / effective_votes).into()
    }

    pub fn absolute_majority_reached(&self) -> bool {
        self.votes_yes * 2 > self.total_voting_power
    }

    /// Try to finalize the vote result.
    /// Returns true if is finalized.
    /// # Panics
    /// Panics if voting_end_time or passing_threshold is None(should never happen).
    pub fn try_finalize_vote_result(&mut self) -> Result<bool, ProposalError> {
        #![allow(clippy::unwrap_used)]
        match self.is_expired() {
            // Voting not finished, try finalize
            false => {
                if self.current_participation_rate()
                    >= self.passing_threshold.clone().unwrap().quorum
                    && self.absolute_majority_reached()
                {
                    self.state_transition(ProposalState::Accepted)?;
                    self.finalize_activation();
                    self.finalize_expiration();
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            // Voting finished, must finalize
            true => {
                if self.current_participation_rate()
                    < self.passing_threshold.clone().unwrap().quorum
                {
                    self.state_transition(ProposalState::QuorumNotMet)?;
                } else if self.current_yes_rate()
                    < self.passing_threshold.clone().unwrap().passing_threshold
                {
                    self.state_transition(ProposalState::Rejected)?;
                } else {
                    self.state_transition(ProposalState::Accepted)?;
                }
                self.finalize_activation();
                self.finalize_expiration();
                Ok(true)
            }
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Serialize)]
pub enum ProposalState {
    /// The proposal is submitted and waiting for validation. Next states: Open, ValidationFailed.
    Submitted,
    /// The proposal failed validation. END.
    ValidationFailed,
    /// The proposal is validated and open for voting. Next states: Accepted, Rejected, Revoked, QuorumNotMet, ForceExecuting.
    Open,
    /// Enough "yes" votes have been cast to accept the proposal, and it will soon be executed. Next states: Executing.
    Accepted,
    /// The proposal is currently being executed. Next states: Succeeded, Failed, Expired.
    Executing(ExecutionStep),
    /// The proposal has been successfully executed. END.
    Succeeded,
    /// A failure occurred while executing the proposal. END.
    Failed(ExecutionStep),
    /// The proposal has expired without being executed. END.
    Expired,
    /// Enough "no" votes have been cast to reject the proposal, and it will not be executed. END.
    Rejected,
    /// Revoked during voting process. END.
    Revoked,
    /// The quorum was not met during the voting period. END.
    QuorumNotMet,
    /// The proposal was force executed. Next states: ForceExecutionSucceeded, ForceExecutionFailed.
    ForceExecuting(ExecutionStep),
    /// The proposal was successfully force executed. END.
    ForceExecutionSucceeded,
    /// A failure occurred while force executing the proposal. END.
    ForceExecutionFailed(ExecutionStep),
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ProposalMetadata {
    /// Name of the proposal.
    pub name: String,
    /// Proposal description.
    pub description: String,
    /// Custom memo from proposer. Not used anywhere in the governance system.
    pub memo: RawBytes,
}

impl Validate for ProposalMetadata {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.description.is_empty()
    }
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ProposalRevoke {
    /// The proposal id.
    pub proposal_id: Index,
    /// The reason of the proposal being revoked.
    pub reason: String,
    /// Timestamp when the proposal was revoked.
    pub revoked_at: TimeNs,
}

#[derive(CandidType, Serialize, Default, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ExecutionStepState {
    /// The default state. Next states: `PreValidating`.
    #[default]
    NotStarted,
    /// Pre-validation. Next states: `PreValidateCallError`, `PreValidateFailed`, `Executing`.
    PreValidating,
    /// Canister call error in pre-validation. END.
    PreValidateCallError,
    /// Prevalidate returned false. END.
    PreValidateFailed,
    /// Prevalidate returned true. Executing the payload. Next states: `ExecutionCallError`, `PostValidating`.
    Executing,
    /// Canister call error in payload execution. END.
    ExecutionCallError,
    /// Payload execution completed. Post-validation. Next states: `PostValidateCallError`, `PostValidateFailed`, `PostValidatePassed`.
    PostValidating,
    /// Canister call error in post-validation. END.
    PostValidateCallError,
    /// Post-validate returned false. END.
    PostValidateFailed,
    /// Post-validate returned true. END.
    Succeeded,
}

#[derive(Clone, Default, Debug, CandidType, Deserialize, PartialEq, Serialize)]
pub struct ExecutionStep {
    /// Index of current step.
    pub step: u8,
    /// State within the current step.
    pub state: ExecutionStepState,
}

#[allow(clippy::enum_variant_names)]
#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ExecutionStepError {
    GenericError,
    StateTransitionError,
    ArithmeticError,
}

impl ExecutionStep {
    pub fn new(step: u8) -> Self {
        Self {
            step,
            state: ExecutionStepState::NotStarted,
        }
    }

    /// Ensures proper state transitions of the state machine.
    /// Returns previous state if transition successful.
    pub fn state_transition(
        &mut self,
        next_state: ExecutionStepState,
    ) -> Result<ExecutionStepState, ExecutionStepError> {
        match self.state {
            ExecutionStepState::NotStarted => match next_state {
                ExecutionStepState::PreValidating => {
                    self.state = next_state;
                    Ok(ExecutionStepState::NotStarted)
                }
                _ => Err(ExecutionStepError::StateTransitionError),
            },
            ExecutionStepState::PreValidating => match next_state {
                ExecutionStepState::PreValidateCallError => {
                    self.state = next_state;
                    Ok(ExecutionStepState::PreValidating)
                }
                ExecutionStepState::PreValidateFailed => {
                    self.state = next_state;
                    Ok(ExecutionStepState::PreValidating)
                }
                ExecutionStepState::Executing => {
                    self.state = next_state;
                    Ok(ExecutionStepState::PreValidating)
                }
                _ => Err(ExecutionStepError::StateTransitionError),
            },
            ExecutionStepState::Executing => match next_state {
                ExecutionStepState::ExecutionCallError => {
                    self.state = next_state;
                    Ok(ExecutionStepState::Executing)
                }
                ExecutionStepState::PostValidating => {
                    self.state = next_state;
                    Ok(ExecutionStepState::Executing)
                }
                _ => Err(ExecutionStepError::StateTransitionError),
            },
            ExecutionStepState::PostValidating => match next_state {
                ExecutionStepState::PostValidateCallError => {
                    self.state = next_state;
                    Ok(ExecutionStepState::PostValidating)
                }
                ExecutionStepState::PostValidateFailed => {
                    self.state = next_state;
                    Ok(ExecutionStepState::PostValidating)
                }
                ExecutionStepState::Succeeded => {
                    self.state = next_state;
                    Ok(ExecutionStepState::PostValidating)
                }
                _ => Err(ExecutionStepError::StateTransitionError),
            },
            ExecutionStepState::PreValidateCallError
            | ExecutionStepState::PreValidateFailed
            | ExecutionStepState::ExecutionCallError
            | ExecutionStepState::PostValidateCallError
            | ExecutionStepState::PostValidateFailed
            | ExecutionStepState::Succeeded => Err(ExecutionStepError::StateTransitionError),
        }
    }
}
