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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{State, VoteRecord};
    use rand::Rng;
    fn generate_random_principal() -> Principal {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 29] = rng.gen(); // Generate 29 random bytes
        Principal::from_slice(&random_bytes)
    }

    #[test]
    fn test_config() {
        // The config is initialized with default values
        let config = get_config().unwrap();
        assert_eq!(config.name, "multisig-voting");
        assert_eq!(config.description, "An m-of-n multisig canister");
        assert!(!config.initialized);
        assert_eq!(config.governance_canister, Principal::anonymous());
        assert_eq!(config.votes_required, 2);
        assert_eq!(config.total_votes, 3);
        assert_eq!(config.vote_buffer_time, 900 * 1_000_000_000);
    }

    #[test]
    fn test_init() {
        config_set_initialized();
        assert!(config_is_initialized());
        let config = get_config().unwrap();
        assert!(config.initialized);
    }

    #[test]
    fn test_config_set_governance() {
        let principal = generate_random_principal();
        config_set_governance(principal);
        let config = get_config().unwrap();
        assert_eq!(config.governance_canister, principal);
    }

    #[test]
    fn test_config_set_name_description() {
        config_set_name_description("test-name".to_string(), "test-description".to_string());
        let config = get_config().unwrap();
        assert_eq!(config.name, "test-name");
        assert_eq!(config.description, "test-description");
    }

    #[test]
    fn test_config_set_m_of_n() {
        config_set_m_of_n(3, 4);
        let config = get_config().unwrap();
        assert_eq!(config.votes_required, 3);
        assert_eq!(config.total_votes, 4);
    }

    #[test]
    fn test_proposal_state() {
        let index = 1;
        let expir = 1000u64;
        let state_open = ProposalState {
            expiration: expir,
            state: State::Open,
            vote_record: VoteRecord {
                yes_votes: vec![],
                no_votes: vec![],
                abstain_votes: vec![],
            },
        };
        let state_failed = ProposalState {
            expiration: expir,
            state: State::Failed,
            vote_record: VoteRecord {
                yes_votes: vec![],
                no_votes: vec![],
                abstain_votes: vec![],
            },
        };
        add_proposal_state(index, state_open.clone());
        let state_res = get_proposal_state(index).unwrap();
        assert_eq!(state_open, state_res);

        add_proposal_state(index, state_failed.clone()); // no overwrite
        let state_res = get_proposal_state(index).unwrap();
        assert_eq!(state_open, state_res);

        set_proposal_state(index, state_failed.clone()); // overwrite
        let state_res = get_proposal_state(index).unwrap();
        assert_eq!(state_failed, state_res);

        add_proposal_state(index + 1, state_failed.clone());
        let state_res = get_proposal_state(index + 1).unwrap();
        assert_eq!(state_failed, state_res);
    }
}
