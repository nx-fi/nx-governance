use crate::access::*;
use crate::memory::*;
use crate::types::*;

use candid::Principal;
use ic_cdk_macros::init;

#[init]
pub fn init(gov_main_principal: Principal) {
    #[allow(clippy::expect_used)]
    add_admin_during_init([gov_main_principal].to_vec()).expect("admin init failed");

    // Other init code here

    // Change initialized to true
    CONFIG.with(|c| {
        let mut config = c.borrow_mut();
        let _ = config.set(Cbor(Some(Config {
            name: "test-canister".to_string(),
            description: "A test canister for NX Governance".to_string(),
            initialized: false,
            counter: 0,
        })));
    });
}
