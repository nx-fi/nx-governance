mod access;
mod lifecycle;
mod memory;
mod types;

use crate::access::*;
use crate::memory::*;
use crate::types::*;

#[allow(unused_imports)]
use candid::Principal;
use ic_cdk_macros::{query, update};

#[update]
pub fn increment() -> u64 {
    CONFIG.with(|c| {
        let mut config = c.borrow_mut();
        let mut c = config.get().0.clone().unwrap();
        c.counter += 1;
        let _ = config.set(Cbor(Some(c.clone())));
        c.counter
    })
}

#[update]
pub fn admin_change_counter(new_counter: u64) -> u64 {
    require_caller_has_role(UserRole::Admin);
    CONFIG.with(|c| {
        let mut config = c.borrow_mut();
        let mut c = config.get().0.clone().unwrap();
        c.counter = new_counter;
        let _ = config.set(Cbor(Some(c.clone())));
        c.counter
    })
}

#[query]
pub fn get_counter() -> u64 {
    CONFIG.with(|c| c.borrow().get().0.clone().unwrap().counter)
}

#[query]
pub fn get_config() -> Config {
    CONFIG.with(|c| c.borrow().get().0.clone().unwrap())
}

#[query]
pub fn get_name() -> String {
    CONFIG.with(|c| c.borrow().get().0.clone().unwrap().name)
}

#[query]
pub fn get_description() -> String {
    CONFIG.with(|c| c.borrow().get().0.clone().unwrap().description)
}

#[query]
pub fn is_initialized() -> bool {
    CONFIG.with(|c| c.borrow().get().0.clone().unwrap().initialized)
}

#[cfg(any(target_arch = "wasm32", test))]
ic_cdk::export_candid!();

fn main() {}
