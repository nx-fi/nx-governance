use crate::memory::{
    ADMIN_ROLES, EXECUTOR_ROLES, FORCE_EXECUTOR_ROLES, PROPOSER_ROLES, REVOKER_ROLES,
    VALIDATOR_ROLES, VOTE_MANAGER_ROLES,
};
use crate::storage::StablePrincipal;
use crate::types::{ReturnError, VM};

use candid::{CandidType, Principal};
use ic_cdk_macros::{query, update};
use ic_stable_structures::StableVec;
use serde::Deserialize;
use std::cell::RefCell;

#[derive(Clone, Debug, CandidType, Deserialize)]
#[repr(u8)]
pub enum UserRole {
    /// Admins can add and remove roles, and perform other priviledged actions (to be controlled via proposals).
    Admin = 0,
    /// Proposers can submit proposals.
    Proposer = 1,
    /// VoteManagers can send in voting results on proposals.
    VoteManager = 2,
    /// Revokers can revoke proposals any time during the voting period.
    Revoker = 3,
    /// Executors can execute proposals after the vote has passed.
    Executor = 4,
    /// ForceExecutors can force execute proposals at any time during the voting period.
    ForceExecutor = 5,
    /// Validators can validate proposals, including assigning the proper proposal type.
    Validator = 6,
}

#[update]
pub fn add_role(role: UserRole, principal: Principal) -> Result<(), ReturnError> {
    require_caller_has_role(UserRole::Admin);
    add_role_internal(role, principal)
}

pub(crate) fn add_role_internal(role: UserRole, principal: Principal) -> Result<(), ReturnError> {
    let op = |roles: &RefCell<StableVec<StablePrincipal, VM>>| -> Result<(), ReturnError> {
        if roles
            .borrow()
            .iter()
            .any(|p| p == StablePrincipal::from(principal))
        {
            return Err(ReturnError::AlreadyExists);
        }
        roles
            .borrow_mut()
            .push(&StablePrincipal::from(principal))
            .map_err(|_| ReturnError::MemoryError)
    };
    match role {
        UserRole::Admin => ADMIN_ROLES.with(op),
        UserRole::Proposer => PROPOSER_ROLES.with(op),
        UserRole::VoteManager => VOTE_MANAGER_ROLES.with(op),
        UserRole::Revoker => REVOKER_ROLES.with(op),
        UserRole::Executor => EXECUTOR_ROLES.with(op),
        UserRole::ForceExecutor => FORCE_EXECUTOR_ROLES.with(op),
        UserRole::Validator => VALIDATOR_ROLES.with(op),
    }
}

#[update]
pub fn remove_role(role: UserRole, principal: Principal) {
    require_caller_has_role(UserRole::Admin);
    let op = |roles: &RefCell<StableVec<StablePrincipal, VM>>| {
        if let Some(index) = roles
            .borrow()
            .iter()
            .position(|p| p == StablePrincipal::from(principal))
        {
            let r = roles.borrow_mut();
            r.set(index as u64, &r.get(r.len() - 1).unwrap());
            r.pop();
        }
    };
    match role {
        UserRole::Admin => ADMIN_ROLES.with(op),
        UserRole::Proposer => PROPOSER_ROLES.with(op),
        UserRole::VoteManager => VOTE_MANAGER_ROLES.with(op),
        UserRole::Revoker => REVOKER_ROLES.with(op),
        UserRole::Executor => EXECUTOR_ROLES.with(op),
        UserRole::ForceExecutor => FORCE_EXECUTOR_ROLES.with(op),
        UserRole::Validator => VALIDATOR_ROLES.with(op),
    };
}

#[update]
pub fn clear_users_of_role(role: UserRole) {
    require_caller_has_role(UserRole::Admin);
    let op = |roles: &RefCell<StableVec<StablePrincipal, VM>>| {
        let r = roles.borrow_mut();
        for _ in 0..r.len() {
            r.pop();
        }
    };
    match role {
        UserRole::Admin => ADMIN_ROLES.with(op),
        UserRole::Proposer => PROPOSER_ROLES.with(op),
        UserRole::VoteManager => VOTE_MANAGER_ROLES.with(op),
        UserRole::Revoker => REVOKER_ROLES.with(op),
        UserRole::Executor => EXECUTOR_ROLES.with(op),
        UserRole::ForceExecutor => FORCE_EXECUTOR_ROLES.with(op),
        UserRole::Validator => VALIDATOR_ROLES.with(op),
    };
}

#[query]
pub fn has_role(role: UserRole, principal: Principal) -> bool {
    let op = |roles: &RefCell<StableVec<StablePrincipal, VM>>| {
        roles
            .borrow()
            .iter()
            .any(|p| p == StablePrincipal::from(principal))
    };
    match role {
        UserRole::Admin => ADMIN_ROLES.with(op),
        UserRole::Proposer => PROPOSER_ROLES.with(op),
        UserRole::VoteManager => VOTE_MANAGER_ROLES.with(op),
        UserRole::Revoker => REVOKER_ROLES.with(op),
        UserRole::Executor => EXECUTOR_ROLES.with(op),
        UserRole::ForceExecutor => FORCE_EXECUTOR_ROLES.with(op),
        UserRole::Validator => VALIDATOR_ROLES.with(op),
    }
}

pub fn require_caller_has_role(role: UserRole) {
    if !has_role(role, ic_cdk::caller()) {
        ic_cdk::trap("Caller does not have role");
    }
}

#[query]
pub fn users_of_role(role: UserRole) -> Vec<Principal> {
    let op = |roles: &RefCell<StableVec<StablePrincipal, VM>>| {
        roles.borrow().iter().map(|p| p.into()).collect()
    };
    match role {
        UserRole::Admin => ADMIN_ROLES.with(op),
        UserRole::Proposer => PROPOSER_ROLES.with(op),
        UserRole::VoteManager => VOTE_MANAGER_ROLES.with(op),
        UserRole::Revoker => REVOKER_ROLES.with(op),
        UserRole::Executor => EXECUTOR_ROLES.with(op),
        UserRole::ForceExecutor => FORCE_EXECUTOR_ROLES.with(op),
        UserRole::Validator => VALIDATOR_ROLES.with(op),
    }
}
