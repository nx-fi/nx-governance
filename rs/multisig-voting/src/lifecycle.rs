use crate::access::*;
use crate::memory::*;

use candid::Principal;
use ic_cdk_macros::{init, update};

#[init]
pub fn init(gov_main_principal: Principal, init_signers: Vec<Principal>) {
    #[allow(clippy::expect_used)]
    add_role_internal(UserRole::Admin, gov_main_principal).expect("admin init failed");
    config_set_governance(gov_main_principal);

    // Other init code here
    SIGNERS.with(|s| {
        let mut s = s.borrow_mut();
        for signer in init_signers {
            s.insert(signer.into(), ());
        }
    });
}

#[update]
pub fn initialize() {
    assert!(ic_cdk::api::is_controller(&ic_cdk::api::caller()));
    assert!(!config_is_initialized());

    // initialization code

    config_set_initialized();
}
