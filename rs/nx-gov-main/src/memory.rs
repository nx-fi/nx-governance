use crate::execution::*;
use crate::proposal::*;
use crate::storage::*;
use crate::types::*;

use ic_cdk_macros::query;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, StableLog, StableVec};
use std::cell::RefCell;

const CONFIG_PAGE_SIZE: u64 = 1;

const CONFIG_PAGE_START: u64 = 0;
const CONFIG_PAGE_END: u64 = CONFIG_PAGE_START + CONFIG_PAGE_SIZE;

const MM_PAGE_START: u64 = 512;

// Managed stable memory
const ADMIN_ROLES_MEM_ID: MemoryId = MemoryId::new(0);
const PROPOSER_ROLES_MEM_ID: MemoryId = MemoryId::new(1);
const VOTE_MANAGER_ROLES_MEM_ID: MemoryId = MemoryId::new(2);
const REVOKER_ROLES_MEM_ID: MemoryId = MemoryId::new(3);
const EXECUTOR_ROLES_MEM_ID: MemoryId = MemoryId::new(4);
const FORCE_EXECUTOR_ROLES_MEM_ID: MemoryId = MemoryId::new(5);
const VALIDATOR_ROLES_MEM_ID: MemoryId = MemoryId::new(6);
const PROPOSALS_MEM_ID: MemoryId = MemoryId::new(7);
const PROPOSAL_EXEC_MEM_ID: MemoryId = MemoryId::new(8);
const TIMER_TASKS_MEM_ID: MemoryId = MemoryId::new(9);

const PROPOSAL_METADATA_LOG_INDEX_MEM_ID: MemoryId = MemoryId::new(60);
const PROPOSAL_METADATA_LOG_DATA_MEM_ID: MemoryId = MemoryId::new(61);
const PROPOSAL_PAYLOAD_LOG_INDEX_MEM_ID: MemoryId = MemoryId::new(62);
const PROPOSAL_PAYLOAD_LOG_DATA_MEM_ID: MemoryId = MemoryId::new(63);
const PROPOSAL_REVOKE_LOG_INDEX_MEM_ID: MemoryId = MemoryId::new(64);
const PROPOSAL_REVOKE_LOG_DATA_MEM_ID: MemoryId = MemoryId::new(65);

thread_local! {
    pub static CONFIG: RefCell<StableCell<Cbor<Option<Config>>, RM>> =
        #[allow(clippy::expect_used)]
        RefCell::new(StableCell::init(
            RM::new(DefaultMemoryImpl::default(), CONFIG_PAGE_START..CONFIG_PAGE_END),
            Cbor(Some(Config {
                name: "nx-gov".to_string(),
                description: "A governance framework on the Internet Computer.".to_string(),
                initialized: false,
                min_voting_period: 86400 * 1_000_000_000 * 3,
                min_passing_threshold: ProposalPassingThreshold::default(),
                voting_may_end_early: true,
                validator_hook: None,
                vote_manager_hook: None,
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

    pub static PROPOSER_ROLES: RefCell<StableVec<StablePrincipal, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableVec::init(
                mm.borrow().get(PROPOSER_ROLES_MEM_ID)).expect("init failed"))
    });

    pub static VOTE_MANAGER_ROLES: RefCell<StableVec<StablePrincipal, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableVec::init(
                mm.borrow().get(VOTE_MANAGER_ROLES_MEM_ID)).expect("init failed"))
    });

    pub static REVOKER_ROLES: RefCell<StableVec<StablePrincipal, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableVec::init(
                mm.borrow().get(REVOKER_ROLES_MEM_ID)).expect("init failed"))
    });

    pub static EXECUTOR_ROLES: RefCell<StableVec<StablePrincipal, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableVec::init(
                mm.borrow().get(EXECUTOR_ROLES_MEM_ID)).expect("init failed"))
    });

    pub static FORCE_EXECUTOR_ROLES: RefCell<StableVec<StablePrincipal, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableVec::init(
                mm.borrow().get(FORCE_EXECUTOR_ROLES_MEM_ID)).expect("init failed"))
    });

    pub static VALIDATOR_ROLES: RefCell<StableVec<StablePrincipal, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableVec::init(
                mm.borrow().get(VALIDATOR_ROLES_MEM_ID)).expect("init failed"))
    });

    pub static PROPOSALS: RefCell<StableVec<Proposal, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableVec::init(
                mm.borrow().get(PROPOSALS_MEM_ID)).expect("init failed"))
    });

    pub static PROPOSAL_METADATA: RefCell<StableLog<ProposalMetadata, VM, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableLog::init(
                mm.borrow().get(PROPOSAL_METADATA_LOG_INDEX_MEM_ID),
                mm.borrow().get(PROPOSAL_METADATA_LOG_DATA_MEM_ID)).expect("init failed"))
    });

    pub static PROPOSAL_PAYLOAD: RefCell<StableLog<ProposalPayload, VM, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableLog::init(
                mm.borrow().get(PROPOSAL_PAYLOAD_LOG_INDEX_MEM_ID),
                mm.borrow().get(PROPOSAL_PAYLOAD_LOG_DATA_MEM_ID)).expect("init failed"))
    });

    pub static PROPOSAL_EXEC: RefCell<StableBTreeMap<Index, ProposalExec, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableBTreeMap::init(
                mm.borrow().get(PROPOSAL_EXEC_MEM_ID)))
    });

    pub static PROPOSAL_REVOKE: RefCell<StableLog<ProposalRevoke, VM, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableLog::init(
                mm.borrow().get(PROPOSAL_REVOKE_LOG_INDEX_MEM_ID),
                mm.borrow().get(PROPOSAL_REVOKE_LOG_DATA_MEM_ID)).expect("init failed"))
    });

    // Proposal IDs that have push notifications. LIFO.
    pub static TIMER_TASKS: RefCell<StableVec<Index, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableVec::init(
                mm.borrow().get(TIMER_TASKS_MEM_ID)).expect("init failed"))
    });

}

// ==== Config ====
#[query]
pub fn get_config() -> Result<Config, ReturnError> {
    CONFIG.with(|c| c.borrow().get().0.clone().ok_or(ReturnError::MemoryError))
}

pub(crate) fn config_set_initialized() {
    CONFIG.with(|c| {
        let mut config = c.borrow().get().0.clone().unwrap();
        config.initialized = true;
        let _ = c.borrow_mut().set(Cbor(Some(config)));
    });
}

pub(crate) fn config_is_initialized() -> bool {
    CONFIG.with(|c| c.borrow().get().0.clone().unwrap().initialized)
}

// ==== Proposals ====
#[query]
pub fn get_all_open_proposal_ids_with_expiration() -> Vec<(Index, TimeNs)> {
    PROPOSALS.with(|p| {
        p.borrow()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if p.state == ProposalState::Open {
                    Some((i as Index, p.voting_end_time.unwrap()))
                } else {
                    None
                }
            })
            .collect()
    })
}

#[query]
pub fn get_all_submitted_proposal_ids() -> Vec<Index> {
    PROPOSALS.with(|p| {
        p.borrow()
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if p.state == ProposalState::Submitted {
                    Some(i as Index)
                } else {
                    None
                }
            })
            .collect()
    })
}

#[query]
pub fn get_proposal_states(skip: Index, take: Index) -> Vec<ProposalState> {
    PROPOSALS.with(|p| {
        p.borrow()
            .iter()
            .skip(skip as usize)
            .take(take as usize)
            .map(|p| p.state.clone())
            .collect()
    })
}

#[query]
pub fn get_next_proposal_id() -> Index {
    PROPOSALS.with(|p| p.borrow().len() as Index)
}

#[query]
pub fn get_proposal(proposal_id: Index) -> Option<Proposal> {
    PROPOSALS.with(|p| p.borrow().get(proposal_id))
}

pub(crate) fn get_proposal_by_id(proposal_id: Index) -> Result<Proposal, ReturnError> {
    PROPOSALS.with(|p| p.borrow().get(proposal_id).ok_or(ReturnError::InvalidIndex))
}

pub(crate) fn add_proposal(proposal: &Proposal) -> Result<u64, ReturnError> {
    PROPOSALS.with(|p| {
        let p = p.borrow_mut();
        p.push(proposal).map_err(|_| ReturnError::MemoryError)?;
        Ok(p.len() - 1)
    })
}

pub(crate) fn set_proposal_by_id(proposal_id: Index, proposal: &Proposal) {
    PROPOSALS.with(|p| p.borrow_mut().set(proposal_id, proposal))
}

// ==== ProposalMetadata ====
pub(crate) fn add_proposal_metadata(metadata: &ProposalMetadata) -> Result<u64, ReturnError> {
    PROPOSAL_METADATA.with(|p| {
        p.borrow_mut()
            .append(metadata)
            .map_err(|_| ReturnError::MemoryError)
    })
}

#[query]
pub fn get_proposal_metadata(proposal_id: Index) -> Option<ProposalMetadata> {
    PROPOSAL_METADATA.with(|p| p.borrow().get(proposal_id))
}

// ==== ProposalPayload ====
pub(crate) fn add_proposal_payload(payload: &ProposalPayload) -> Result<u64, ReturnError> {
    PROPOSAL_PAYLOAD.with(|p| {
        p.borrow_mut()
            .append(payload)
            .map_err(|_| ReturnError::MemoryError)
    })
}

#[query]
pub fn get_proposal_payload(proposal_id: Index) -> Option<ProposalPayload> {
    PROPOSAL_PAYLOAD.with(|p| p.borrow().get(proposal_id))
}

pub(crate) fn get_proposal_payload_by_id(
    payload_id: Index,
) -> Result<ProposalPayload, ReturnError> {
    PROPOSAL_PAYLOAD.with(|p| {
        p.borrow()
            .get(payload_id)
            .ok_or(ReturnError::InvalidIndex)
            .map(|p| p.clone())
    })
}

// ==== ProposalRevoke ====
pub(crate) fn add_proposal_revoke(revoke_data: &ProposalRevoke) -> Result<u64, ReturnError> {
    PROPOSAL_REVOKE.with(|p| {
        p.borrow_mut()
            .append(revoke_data)
            .map_err(|_| ReturnError::MemoryError)
    })
}

#[query]
pub fn get_proposal_revoke(revoke_id: Index) -> Result<ProposalRevoke, ReturnError> {
    PROPOSAL_REVOKE.with(|p| {
        p.borrow()
            .get(revoke_id)
            .ok_or(ReturnError::InvalidIndex)
            .map(|p| p.clone())
    })
}

// ==== ProposalExec ====
#[allow(dead_code)]
pub(crate) fn set_execution_result(id: Index, proposal_exe_result: ProposalExec) {
    PROPOSAL_EXEC.with(|p| p.borrow_mut().insert(id, proposal_exe_result));
}

pub(crate) fn add_execution_step_result(id: Index, step_result: ExecResult) {
    PROPOSAL_EXEC.with(|p| {
        let mut proposal_exe_result = p.borrow_mut().get(&id).unwrap_or_default();
        proposal_exe_result.execution_result.push(step_result);
        p.borrow_mut().insert(id, proposal_exe_result);
    });
}

#[query]
pub fn get_proposal_execution_result(id: Index) -> Result<ProposalExec, ReturnError> {
    PROPOSAL_EXEC.with(|p| {
        p.borrow()
            .get(&id)
            .ok_or(ReturnError::InvalidIndex)
            .map(|p| p.clone())
    })
}

// ==== TimerTasks ====
pub(crate) fn push_timer_task(proposal_id: Index) -> Result<(), ReturnError> {
    TIMER_TASKS.with(|t| {
        t.borrow_mut()
            .push(&proposal_id)
            .map_err(|_| ReturnError::MemoryError)
    })
}

#[allow(dead_code)]
pub(crate) fn pop_timer_task() -> Option<Index> {
    TIMER_TASKS.with(|t| t.borrow_mut().pop())
}
