use crate::types::*;
use crate::validate::Validate;

use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

/// The execution may depend on a pre-validation step by an external canister.
///
/// This execution validation canister may be different from the validation canister,
/// and can be different for each `CanisterMessage`.
/// But the canister must be safe.
/// This can be guaranteed by a whitelist in the validation canister.
///
/// The pre-validation target can be the execution target canister, the governance canister itself, or any other canister.
///
/// The pre-validation should be used with caution, because the Internet Computer is asynchronous,
/// and the state may change between when the pre-validation is executed and when the execution payload is executed.
/// However it is still useful in some cases where state changes are controlled (for example under NX Governance).
/// The return type of the validation method must be `bool`. If it returns `false`, the execution not continue.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct PreValidateTarget {
    /// Canister id of the execution validation canister.
    pub canister_id: Principal,
    /// Method name of the execution validation canister.
    pub method: String,
    /// Payload of the execution validation canister.
    pub payload: RawBytes,
    /// Payment in cycles for the execution validation canister.
    pub payment: u128,
}

/// The execution result can be validated by an external canister.
///
/// Note that the validation canister has the entire input and result of the execution payload, and can perform further async calls with it.
/// This can be dangerous but also gives more flexibility.
/// For example, the execution payload can be a call to the management canister, and the result can be used in the validation canister.
///
/// This execution validation canister may be different from the validation canister,
/// and can be different for each `CanisterMessage`.
/// But the canister must be safe.
/// This can be guaranteed by a whitelist in the validation canister.
/// The payload does not need to be specified, it is always `PostValidatePayload`.
/// The return type of the validation method must be `bool`. If it returns `false`, the execution not continue.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct PostValidateTarget {
    /// Canister id of the execution validation canister.
    pub canister_id: Principal,
    /// Method name of the execution validation canister.
    pub method: String,
    /// Payment in cycles for the execution validation canister.
    pub payment: u128,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct PostValidatePayload {
    /// Canister id of the execution payload.
    pub canister_id: Principal,
    /// Method name of the execution payload.
    pub method: String,
    /// Message of the execution payload.
    pub message: RawBytes,
    /// Response of the execution payload (CallResult already unwrapped).
    pub response: RawBytes,
}

/// The execution payload, for a single canister call with pre-validation and post-validation.
///
/// One pre-validate and one post-validate can be specified for each message.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct CanisterMessage {
    /// Canister id of the execution payload.
    pub canister_id: Principal,
    /// Method name of the execution payload.
    pub method: String,
    /// Message of the execution payload.
    pub message: RawBytes,
    /// Payment in cycles for the execution payload.
    pub payment: u128,
    pub pre_validate: Option<PreValidateTarget>,
    pub post_validate: Option<PostValidateTarget>,
}

/// Messages are to be executed sequentially.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ProposalPayload {
    /// Ids of proposals that this proposal depends on.
    /// All the depends_on proposals must be executed successfully before this proposal can be executed.
    /// All the referenced proposal ids must be less than this proposal id.
    /// If the vector is empty, it means this proposal does not depend on any other proposal.
    /// ForceExecute cannot bypass this dependency.
    pub depends_on: Vec<Index>,
    /// Messages to be executed.
    pub messages: Vec<CanisterMessage>,
}

impl Validate for ProposalPayload {
    fn is_valid(&self) -> bool {
        // all method must not be empty
        self.messages.iter().all(|m| !m.method.is_empty())
    }
}

impl ProposalPayload {
    pub fn max_dependency_index(&self) -> Option<Index> {
        self.depends_on.iter().max().copied()
    }
}

/// The raw type of `ic_cdk::api::call::CallResult`, using `RawBytes` and `i32`.
/// Can be decoded using `candid:decode_args`.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ExecResult(pub Result<RawBytes, (i32, String)>);

/// Result of the execution of a proposal.
/// Each `CanisterMessage` produces one `ExecResult`.
#[derive(Clone, Debug, Default, CandidType, Deserialize, Serialize)]
pub struct ProposalExec {
    /// The execution result of the proposal.
    pub execution_result: Vec<ExecResult>,
}
