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
    _remove_role(role, principal);
}

fn _remove_role(role: UserRole, principal: Principal) {
    let op = |roles: &RefCell<StableVec<StablePrincipal, VM>>| {
        let index = {
            roles
                .borrow()
                .iter()
                .position(|p| p == StablePrincipal::from(principal))
        };
        if let Some(index) = index {
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
    _clear_users_of_role(role)
}

fn _clear_users_of_role(role: UserRole) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    fn generate_random_principal() -> Principal {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 29] = rng.gen(); // Generate 29 random bytes
        Principal::from_slice(&random_bytes)
    }

    #[test]
    fn role_test() -> Result<(), String> {
        let n = 3;
        let mut principal_admin: Principal = Principal::anonymous();
        let mut principal_proposer: Principal = Principal::anonymous();
        let mut principal_vote_manager: Principal = Principal::anonymous();
        let mut principal_revoker: Principal = Principal::anonymous();
        let mut principal_executor: Principal = Principal::anonymous();
        let mut principal_force_executor: Principal = Principal::anonymous();
        let mut principal_validator: Principal = Principal::anonymous();

        for _i in 0..n {
            principal_admin = generate_random_principal();
            assert!(!has_role(UserRole::Admin, principal_admin));
            let _res = add_role_internal(UserRole::Admin, principal_admin);
            assert!(has_role(UserRole::Admin, principal_admin));

            principal_proposer = generate_random_principal();
            assert!(!has_role(UserRole::Proposer, principal_proposer));
            let _res = add_role_internal(UserRole::Proposer, principal_proposer);
            assert!(has_role(UserRole::Proposer, principal_proposer));

            principal_vote_manager = generate_random_principal();
            assert!(!has_role(UserRole::VoteManager, principal_vote_manager));
            let _res = add_role_internal(UserRole::VoteManager, principal_vote_manager);
            assert!(has_role(UserRole::VoteManager, principal_vote_manager));
            principal_revoker = generate_random_principal();
            assert!(!has_role(UserRole::Revoker, principal_revoker));
            let _res = add_role_internal(UserRole::Revoker, principal_revoker);
            assert!(has_role(UserRole::Revoker, principal_revoker));

            principal_executor = generate_random_principal();
            assert!(!has_role(UserRole::Executor, principal_executor));
            let _res = add_role_internal(UserRole::Executor, principal_executor);
            assert!(has_role(UserRole::Executor, principal_executor));

            principal_force_executor = generate_random_principal();
            assert!(!has_role(UserRole::ForceExecutor, principal_force_executor));
            let _res = add_role_internal(UserRole::ForceExecutor, principal_force_executor);
            assert!(has_role(UserRole::ForceExecutor, principal_force_executor));

            principal_validator = generate_random_principal();
            assert!(!has_role(UserRole::Validator, principal_validator));
            let _res = add_role_internal(UserRole::Validator, principal_validator);
            assert!(has_role(UserRole::Validator, principal_validator));
        }

        let _users = users_of_role(UserRole::Admin);
        assert_eq!(_users.len(), n as usize);
        let _users = users_of_role(UserRole::Proposer);
        assert_eq!(_users.len(), n as usize);
        let _users = users_of_role(UserRole::VoteManager);
        assert_eq!(_users.len(), n as usize);
        let _users = users_of_role(UserRole::Revoker);
        assert_eq!(_users.len(), n as usize);
        let _users = users_of_role(UserRole::Executor);
        assert_eq!(_users.len(), n as usize);
        let _users = users_of_role(UserRole::ForceExecutor);
        assert_eq!(_users.len(), n as usize);
        let _users = users_of_role(UserRole::Validator);
        assert_eq!(_users.len(), n as usize);

        _remove_role(UserRole::Admin, principal_admin);
        assert!(!has_role(UserRole::Admin, principal_admin));
        _remove_role(UserRole::Proposer, principal_proposer);
        assert!(!has_role(UserRole::Proposer, principal_proposer));
        _remove_role(UserRole::VoteManager, principal_vote_manager);
        assert!(!has_role(UserRole::VoteManager, principal_vote_manager),);
        _remove_role(UserRole::Revoker, principal_revoker);
        assert!(!has_role(UserRole::Revoker, principal_revoker));
        _remove_role(UserRole::Executor, principal_executor);
        assert!(!has_role(UserRole::Executor, principal_executor));
        _remove_role(UserRole::ForceExecutor, principal_force_executor);
        assert!(!has_role(UserRole::ForceExecutor, principal_force_executor));
        _remove_role(UserRole::Validator, principal_validator);
        assert!(!has_role(UserRole::Validator, principal_validator));

        _remove_role(UserRole::Admin, principal_admin); // Should not panic, no effect

        _clear_users_of_role(UserRole::Admin);
        let _users = users_of_role(UserRole::Admin);
        assert_eq!(_users.len(), 0);
        _clear_users_of_role(UserRole::Proposer);
        let _users = users_of_role(UserRole::Proposer);
        assert_eq!(_users.len(), 0);
        _clear_users_of_role(UserRole::VoteManager);
        let _users = users_of_role(UserRole::VoteManager);
        assert_eq!(_users.len(), 0);
        _clear_users_of_role(UserRole::Revoker);
        let _users = users_of_role(UserRole::Revoker);
        assert_eq!(_users.len(), 0);
        _clear_users_of_role(UserRole::Executor);
        let _users = users_of_role(UserRole::Executor);
        assert_eq!(_users.len(), 0);
        _clear_users_of_role(UserRole::ForceExecutor);
        let _users = users_of_role(UserRole::ForceExecutor);
        assert_eq!(_users.len(), 0);
        _clear_users_of_role(UserRole::Validator);
        let _users = users_of_role(UserRole::Validator);
        assert_eq!(_users.len(), 0);

        Ok(())
    }
}
