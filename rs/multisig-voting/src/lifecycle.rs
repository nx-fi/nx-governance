use crate::access::*;
use crate::memory::*;

use candid::Principal;
use ic_cdk_macros::{init, update};

#[init]
pub fn init(
    gov_main_principal: Principal,
    votes_required: u64,
    total_votes: u64,
    signers: Vec<Principal>,
) {
    assert!(votes_required <= total_votes);
    assert_eq!(total_votes, signers.len() as u64); // SAFETY: Uniqueness of signers is checked in add_role_internal.

    #[allow(clippy::expect_used)]
    add_role_internal(UserRole::Admin, gov_main_principal).expect("admin init failed");

    config_set_m_of_n(votes_required, total_votes);

    signers
        .into_iter()
        .for_each(|p| add_role_internal(UserRole::Signer, p).expect("signer init failed"));
    config_set_governance(gov_main_principal);

    // Other init code here
}

#[update]
pub fn initialize() {
    assert!(ic_cdk::api::is_controller(&ic_cdk::api::caller()));
    assert!(!config_is_initialized());

    // initialization code

    config_set_initialized();
}
