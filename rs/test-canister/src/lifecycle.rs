use crate::access::*;
use crate::memory::*;

use candid::Principal;
use ic_cdk_macros::{init, update};

#[init]
pub fn init(gov_main_principal: Principal) {
    #[allow(clippy::expect_used)]
    add_role_internal(UserRole::Admin, gov_main_principal).expect("admin init failed");

    // Other init code here
}

#[update]
pub fn initialize() {
    assert!(ic_cdk::api::is_controller(&ic_cdk::api::caller()));
    assert!(!config_is_initialized());

    // initialization code

    config_set_initialized();
}
