use crate::validate::Validate;

use candid::{CandidType, Principal};
use ic_stable_structures::{memory_manager::VirtualMemory, DefaultMemoryImpl, RestrictedMemory};
use serde::{Deserialize, Serialize};

pub type RM = RestrictedMemory<DefaultMemoryImpl>;
pub type VM = VirtualMemory<RM>;

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ReturnError {
    GenericError,
    InputError,
    Unauthorized,
    Expired,
    InterCanisterCallError,
    MemoryError,
    ArithmeticError,
    InvalidIndex,
    AlreadyExists,
    IncorrectProposalState,
    StateTransitionError,
    DependentProposalNotSucceeded,
    DependentProposalNotReady,
    PreValidateFailed,
    PostValidateFailed,
    ExecutionFailed,
}

/// nano seconds since UNIX Epoch.
pub type TimeNs = u64;

/// Config of the canister.
#[derive(CandidType, Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub struct Config {
    /// Name of the governance canister.
    pub name: String,
    /// Description of the governance canister.
    pub description: String,
    /// Whether the canister has been initialized. Controller can call initialize exactly once.
    pub initialized: bool,
    /// Minimum voting period, in nano-seconds. Prevents extreme values to be set by validator.
    pub min_voting_period: TimeNs,
    /// Minimum passing threshold. Prevents extreme values to be set by validator.
    pub min_passing_threshold: ProposalPassingThreshold,
    /// Can end vote early if result is known.
    pub voting_may_end_early: bool,
    /// Validator notification hook. If set then a timer is used to notify the validator.
    pub validator_hook: Option<Principal>,
    /// Vote manager notification hook. If set then a timer is used to notify the vote manager.
    pub vote_manager_hook: Option<Principal>,
}

pub type VotingPower = i128; // A negative value nullifies a prior vote.
pub type Index = u64;

pub type RawBytes = Vec<u8>;

// TODO: unused
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Stats {
    config: Config,
    current_time: TimeNs,
    number_of_active_proposals: u64,
    number_of_closed_proposals: u64,
    total_bytes_used: u64,
    scheduled_actions: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProposalPassingThreshold {
    /// The percentage of the vote that need to do a vote action for a proposal to pass or reject.
    pub quorum: Percentage<PercentagePrecision>,
    /// The percentage of yes/(yes+no) for a proposal to pass.
    pub passing_threshold: Percentage<PercentagePrecision>,
}

impl Default for ProposalPassingThreshold {
    fn default() -> Self {
        Self {
            quorum: Percentage::<PercentagePrecision>::from_percent(20),
            passing_threshold: Percentage::<PercentagePrecision>::from_percent(20),
        }
    }
}

impl Validate for ProposalPassingThreshold {
    fn is_valid(&self) -> bool {
        self.quorum.is_valid() && self.passing_threshold.is_valid()
    }
}

impl ProposalPassingThreshold {
    pub fn all_fields_gte(&self, other: &Self) -> bool {
        self.quorum >= other.quorum && self.passing_threshold >= other.passing_threshold
    }
}

/// Schedule is either an absolute time or a relative time.
/// Relative time is always converted into absolute time when the trigger conditions are met.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Hash, PartialEq)]
pub enum Schedule {
    /// Exactly at the given time.
    At(TimeNs),
    /// At conversion time + the given time interval.
    In(TimeNs),
}

impl Default for Schedule {
    fn default() -> Self {
        Self::At(0)
    }
}

impl Schedule {
    pub fn convert_to_absolute(&mut self) {
        match self {
            Self::At(_) => {}
            Self::In(time) => {
                *self = Self::At(ic_cdk::api::time() + *time);
            }
        }
    }
    pub fn is_absolute(&self) -> bool {
        match self {
            Self::At(_) => true,
            Self::In(_) => false,
        }
    }
    pub fn is_in_future(&self) -> bool {
        match self {
            Self::At(time) => *time > ic_cdk::api::time(),
            Self::In(_) => true,
        }
    }
    pub fn to_timestamp(&self) -> Option<TimeNs> {
        match self {
            Self::At(time) => Some(*time),
            Self::In(_) => None,
        }
    }
}

/// Generic percentage representation
#[derive(
    Clone, Debug, Default, CandidType, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Percentage<T>(T);

pub type PercentagePrecision = u16; // Change this if higher precision is desired

/// Percentage representation with ppb precision scaled 4x. 4_000_000_000 = 100%
impl Percentage<u32> {
    pub fn from_percent(percent: u8) -> Self {
        Self(percent as u32 * 40_000_000)
    }
    pub fn from_basis_points(basis_points: u16) -> Self {
        Self(basis_points as u32 * 400_000)
    }
    pub fn from_ppm(ppm: u32) -> Self {
        Self(ppm * 4_000)
    }
    pub fn from_ppb(ppb: u32) -> Self {
        Self(ppb * 4)
    }
}

impl From<f32> for Percentage<u32> {
    fn from(f: f32) -> Self {
        Self((f * 4_000_000_000.0) as u32)
    }
}

impl From<f64> for Percentage<u32> {
    fn from(f: f64) -> Self {
        Self((f * 4_000_000_000.0) as u32)
    }
}

impl Validate for Percentage<u32> {
    fn is_valid(&self) -> bool {
        self.0 <= 4_000_000_000
    }
}

/// Percentage representation with basis point precision scaled 4x. 40_000 = 100%
impl Percentage<u16> {
    pub fn from_percent(percent: u8) -> Self {
        Self(percent as u16 * 400)
    }
    pub fn from_basis_points(basis_points: u16) -> Self {
        Self(basis_points * 4)
    }
    pub fn from_ppm(ppm: u32) -> Self {
        Self((ppm / 25) as u16)
    }
    pub fn from_ppb(ppb: u32) -> Self {
        Self((ppb / 25_000) as u16)
    }
}

impl From<f32> for Percentage<u16> {
    fn from(f: f32) -> Self {
        Self((f * 40_000.0) as u16)
    }
}

impl From<f64> for Percentage<u16> {
    fn from(f: f64) -> Self {
        Self((f * 40_000.0) as u16)
    }
}

impl Validate for Percentage<u16> {
    fn is_valid(&self) -> bool {
        self.0 <= 40_000
    }
}
