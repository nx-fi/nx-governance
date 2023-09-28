use crate::types::{Cbor, Config, Index, ProposalState, ReturnError, StablePrincipal, RM, VM};

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
const SIGNER_ROLES_MEM_ID: MemoryId = MemoryId::new(1);
const PROPOSAL_VOTES_MEM_ID: MemoryId = MemoryId::new(2);

thread_local! {
    pub static CONFIG: RefCell<StableCell<Cbor<Option<Config>>, RM>> =
        #[allow(clippy::expect_used)]
        RefCell::new(StableCell::init(
            RM::new(DefaultMemoryImpl::default(), CONFIG_PAGE_START..CONFIG_PAGE_END),
            Cbor(Some(Config {
                name: "multisig-voting".to_string(),
                description: "An m-of-n multisig canister".to_string(),
                initialized: false,
                governance_canister: Principal::anonymous(),
                votes_required: 2,
                total_votes: 3,
                vote_buffer_time: 900 * 1_000_000_000, // voting here ends 15 minutes early to ensure async update back to governance
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

    pub static SIGNER_ROLES: RefCell<StableVec<StablePrincipal, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableVec::init(
                mm.borrow().get(SIGNER_ROLES_MEM_ID)).expect("init failed"))
    });

    // map of proposal index to proposal state
    pub static PROPOSAL_VOTES: RefCell<StableBTreeMap<Index, Cbor<ProposalState>, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableBTreeMap::init(
                mm.borrow().get(PROPOSAL_VOTES_MEM_ID)))
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

pub fn config_set_initialized() {
    CONFIG.with(|c| {
        let mut config = c.borrow().get().0.clone().unwrap();
        config.initialized = true;
        let _ = c.borrow_mut().set(Cbor(Some(config)));
    });
}

pub fn config_is_initialized() -> bool {
    CONFIG.with(|c| c.borrow().get().0.clone().unwrap().initialized)
}

pub fn config_set_name_description(name: String, description: String) {
    CONFIG.with(|c| {
        let mut c = c.borrow_mut();
        let mut config = c.get().0.clone().unwrap();
        config.name = name;
        config.description = description;
        c.set(Cbor(Some(config))).expect("config update failed");
    });
}

pub fn config_set_m_of_n(m: u64, n: u64) {
    CONFIG.with(|c| {
        let mut c = c.borrow_mut();
        let mut config = c.get().0.clone().unwrap();
        config.votes_required = m;
        config.total_votes = n;
        c.set(Cbor(Some(config))).expect("config update failed");
    });
}

// ==== Proposal Votes ====
#[query]
pub fn get_proposal_state(index: Index) -> Option<ProposalState> {
    PROPOSAL_VOTES.with(|p| p.borrow().get(&index).map(|Cbor(state)| state.clone()))
}

#[allow(unused)]
pub fn set_proposal_state(index: Index, state: ProposalState) {
    PROPOSAL_VOTES.with(|p| {
        p.borrow_mut().insert(index, Cbor(state));
    })
}

// Sets proposal state if not existing.
pub fn add_proposal_state(index: Index, state: ProposalState) {
    PROPOSAL_VOTES.with(|p| {
        let mut p = p.borrow_mut();
        if !p.contains_key(&index) {
            p.insert(index, Cbor(state));
        }
    })
}
