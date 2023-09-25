use crate::access::*;
use crate::memory::*;
use crate::types::*;

use candid::Principal;
use ic_cdk_macros::{init, post_upgrade, query};

#[init]
pub fn init(gov_main_principal: Principal) {
    #[allow(clippy::expect_used)]
    add_admin_during_init([gov_main_principal].to_vec()).expect("admin init failed");

    // Other init code here

    // Change initialized to true
    CONFIG.with(|c| {
        let mut config = c.borrow_mut();
        let _ = config.set(Cbor(Some(Config {
            name: "simple-validator".to_string(),
            description: "A simple validator for NX Governance".to_string(),
            initialized: true,
        })));
    });
}
