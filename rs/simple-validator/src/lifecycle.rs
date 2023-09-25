use crate::access::*;
use crate::memory::*;

use candid::Principal;
use ic_cdk_macros::init;

#[init]
pub fn init(gov_main_principal: Principal) {
    #[allow(clippy::expect_used)]
    add_admin_during_init([gov_main_principal].to_vec()).expect("admin init failed");
    config_set_governance(gov_main_principal);

    // other init code here
    // add self to whitelist
    CALL_TARGET_WHITELIST.with(|c| {
        c.borrow_mut().insert(ic_cdk::api::id().into(), ());
    });
}
