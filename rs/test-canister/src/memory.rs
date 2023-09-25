use crate::types::{Cbor, Config, StablePrincipal, RM, VM};

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::{DefaultMemoryImpl, StableCell, StableVec};
use std::cell::RefCell;

const CONFIG_PAGE_SIZE: u64 = 1;

const CONFIG_PAGE_START: u64 = 0;
const CONFIG_PAGE_END: u64 = CONFIG_PAGE_START + CONFIG_PAGE_SIZE;

const MM_PAGE_START: u64 = 512;

// Managed stable memory
const ADMIN_ROLES_MEM_ID: MemoryId = MemoryId::new(0);

thread_local! {
    pub static CONFIG: RefCell<StableCell<Cbor<Option<Config>>, RM>> =
        #[allow(clippy::expect_used)]
        RefCell::new(StableCell::init(
            RM::new(DefaultMemoryImpl::default(), CONFIG_PAGE_START..CONFIG_PAGE_END),
            Cbor(Some(Config {
                name: "test-canister".to_string(),
                description: "A test canister for NX Governance".to_string(),
                initialized: false,
                counter: 0,
            })),
        ).expect("Failed to initialize config")
    );

    // Managed stable memory
    pub static MEMORY_MANAGER: RefCell<MemoryManager<RM>> = RefCell::new(
        MemoryManager::init(RM::new(DefaultMemoryImpl::default(), MM_PAGE_START..u64::MAX/65536-1))
    );

    pub static ADMIN_ROLES: RefCell<StableVec<StablePrincipal, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableVec::init(
                mm.borrow().get(ADMIN_ROLES_MEM_ID)).expect("init failed"))
    });



}
