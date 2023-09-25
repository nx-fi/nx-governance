mod access;
mod lifecycle;
mod memory;
mod types;

use crate::access::*;
use crate::memory::*;
use crate::types::*;

use candid::Principal;
use ic_cdk_macros::{query, update};

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
