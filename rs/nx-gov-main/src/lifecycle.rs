use crate::access::*;
use crate::memory::*;

use candid::{encode_one, Principal};
use ic_cdk_macros::{init, post_upgrade, update};

#[init]
pub fn init() {
    #![allow(clippy::expect_used)]
    add_role_internal(UserRole::Admin, ic_cdk::api::id()).expect("admin init failed"); // must set itself as admin

    // Other initializations
    add_role_internal(UserRole::Proposer, Principal::anonymous()).expect("add_role failed");
}

#[update]
pub async fn initialize(
    validator: Principal,
    vote_manager: Principal,
    executor: Principal,
    validation_wl: Vec<Principal>,
) {
    assert!(ic_cdk::api::is_controller(&ic_cdk::api::caller()));
    assert!(!config_is_initialized());

    add_role_internal(UserRole::Validator, validator).expect("add_role failed");
    add_role_internal(UserRole::VoteManager, vote_manager).expect("add_role failed");
    add_role_internal(UserRole::Executor, executor).expect("add_role failed");

    // Add initial validation whitelist
    let _ = ic_cdk::api::call::call_raw128(
        validator,
        "add_call_targets_to_whitelist",
        &encode_one(validation_wl).unwrap(),
        0,
    )
    .await;

    config_set_initialized();
}

#[post_upgrade]
fn post_upgrade() {
    // Restart timer
}
