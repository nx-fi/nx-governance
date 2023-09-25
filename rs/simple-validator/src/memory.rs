use crate::types::{Cbor, Config, Index, ReturnError, StablePrincipal, RM, VM};

use candid::Principal;
use ic_cdk_macros::query;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, StableVec};
use std::cell::RefCell;

const CONFIG_PAGE_SIZE: u64 = 1;

const CONFIG_PAGE_START: u64 = 0;
const CONFIG_PAGE_END: u64 = CONFIG_PAGE_START + CONFIG_PAGE_SIZE;

const MM_PAGE_START: u64 = 512;

// Managed stable memory
const ADMIN_ROLES_MEM_ID: MemoryId = MemoryId::new(0);
const CALL_TARGET_WHITELIST_MEM_ID: MemoryId = MemoryId::new(1);
const PROPOSAL_VALIDATIONS_MEM_ID: MemoryId = MemoryId::new(2);

thread_local! {
    pub static CONFIG: RefCell<StableCell<Cbor<Option<Config>>, RM>> =
        #[allow(clippy::expect_used)]
        RefCell::new(StableCell::init(
            RM::new(DefaultMemoryImpl::default(), CONFIG_PAGE_START..CONFIG_PAGE_END),
            Cbor(Some(Config {
                name: "simple-validator".to_string(),
                description: "A simple validator for NX Governance".to_string(),
                initialized: false,
                governance_canister: Principal::anonymous(),
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

    pub static CALL_TARGET_WHITELIST: RefCell<StableBTreeMap<StablePrincipal, (), VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableBTreeMap::init(
                mm.borrow().get(CALL_TARGET_WHITELIST_MEM_ID)))
    });

    // map of proposal index to true/false
    pub static PROPOSAL_VALIDATIONS: RefCell<StableBTreeMap<Index, u8, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableBTreeMap::init(
                mm.borrow().get(PROPOSAL_VALIDATIONS_MEM_ID)))
    });

}

// ==== Config ====
#[query]
pub fn get_config() -> Result<Config, ReturnError> {
    CONFIG.with(|c| c.borrow().get().0.clone().ok_or(ReturnError::MemoryError))
}

pub fn config_set_governance(governance: Principal) {
    CONFIG.with(|c| {
        let mut c = c.borrow_mut();
        let mut config = c.get().0.clone().unwrap();
        config.governance_canister = governance;
        c.set(Cbor(Some(config))).unwrap();
    })
}

// ==== Proposal Validations ====
#[query]
pub fn get_proposal_validation(index: Index) -> Option<u8> {
    PROPOSAL_VALIDATIONS.with(|p| p.borrow().get(&index))
}

#[allow(unused)]
pub fn set_proposal_validation(index: Index, validated: bool) {
    PROPOSAL_VALIDATIONS.with(|p| {
        p.borrow_mut().insert(index, validated as u8);
    })
}

// Sets proposal validation if not existing.
pub fn add_proposal_validation(index: Index, validated: bool) {
    PROPOSAL_VALIDATIONS.with(|p| {
        let mut p = p.borrow_mut();
        if !p.contains_key(&index) {
            p.insert(index, validated as u8);
        }
    })
}
