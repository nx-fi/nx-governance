use crate::memory::{ADMIN_ROLES, CONFIG};
use crate::types::{ReturnError, StablePrincipal, VM};

use candid::{CandidType, Principal};
use ic_cdk_macros::{query, update};
use ic_stable_structures::StableVec;
use serde::Deserialize;
use std::cell::RefCell;

#[derive(Clone, Debug, CandidType, Deserialize)]
#[repr(u8)]
pub enum UserRole {
    /// Admins can add and remove roles, and perform other priviledged actions (is typically the governance canister).
    Admin = 0,
}

#[update]
pub fn add_role(role: UserRole, principal: Principal) -> Result<(), ReturnError> {
    require_caller_has_role(UserRole::Admin);
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
    }
}

/// Make sure this function is only called during init
pub(crate) fn add_admin_during_init(principals: Vec<Principal>) -> Result<(), ReturnError> {
    let initialized = CONFIG.with(|c| c.borrow().get().0.clone().unwrap().initialized);
    assert!(
        !initialized,
        "add_admin_during_init should only be called during init"
    );
    ADMIN_ROLES.with(
        |roles: &RefCell<StableVec<StablePrincipal, VM>>| -> Result<(), ReturnError> {
            for principal in principals {
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
                    .map_err(|_| ReturnError::MemoryError)?;
            }
            Ok(())
        },
    )
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use candid::Principal;

    // panicked at 'msg_caller_size should only be called inside canisters.'
    // #[test]
    // fn test_remove_role() {
    //     let principal = Principal::from_text("3we4s-lyaaa-aaaak-aegrq-cai").unwrap();
    //     let _ = add_role(UserRole::Admin, principal);

    //     assert!(has_role(UserRole::Admin, principal));

    //     remove_role(UserRole::Admin, principal);

    //     assert!(!has_role(UserRole::Admin, principal));
    // }
}
