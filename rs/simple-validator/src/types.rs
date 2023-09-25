use candid::{CandidType, Principal};
use ic_stable_structures::{
    memory_manager::VirtualMemory,
    storable::{Blob, Bound},
    DefaultMemoryImpl, RestrictedMemory, Storable,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

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
    AlreadyExists,
}

/// nano seconds since UNIX Epoch.
pub type TimeNs = u64;

/// Config of the canister.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// Name of the canister.
    pub name: String,
    /// Description of the canister.
    pub description: String,
    /// Whether the canister has been initialized.
    pub initialized: bool,
    /// The principal of the governance canister.
    pub governance_canister: Principal,
}

pub type Index = u64;
pub type RawBytes = Vec<u8>;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct PreValidateTarget {
    /// Canister id of the execution validation canister.
    pub canister_id: Principal,
    /// Method name of the execution validation canister.
    pub method: String,
    /// Payload of the execution validation canister.
    pub payload: RawBytes,
    /// Payment in cycles for the execution validation canister.
    pub payment: u128,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct PostValidateTarget {
    /// Canister id of the execution validation canister.
    pub canister_id: Principal,
    /// Method name of the execution validation canister.
    pub method: String,
    /// Payment in cycles for the execution validation canister.
    pub payment: u128,
}

/// The execution payload, for a single canister call with pre-validation and post-validation.
///
/// One pre-validate and one post-validate can be specified for each message.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct CanisterMessage {
    /// Canister id of the execution payload.
    pub canister_id: Principal,
    /// Method name of the execution payload.
    pub method: String,
    /// Message of the execution payload.
    pub message: RawBytes,
    /// Payment in cycles for the execution payload.
    pub payment: u128,
    pub pre_validate: Option<PreValidateTarget>,
    pub post_validate: Option<PostValidateTarget>,
}

/// Messages are to be executed sequentially.
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ProposalPayload {
    /// Ids of proposals that this proposal depends on.
    /// All the depends_on proposals must be executed successfully before this proposal can be executed.
    /// All the referenced proposal ids must be less than this proposal id.
    /// If the vector is empty, it means this proposal does not depend on any other proposal.
    /// ForceExecute cannot bypass this dependency.
    pub depends_on: Vec<Index>,
    /// Messages to be executed.
    pub messages: Vec<CanisterMessage>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProposalPassingThreshold {
    /// The percentage of the vote that need to do a vote action for a proposal to pass or reject.
    pub quorum: Percentage<PercentagePrecision>,
    /// The percentage of yes/(yes+no) for a proposal to pass.
    pub passing_threshold: Percentage<PercentagePrecision>,
}

#[derive(
    Clone, Debug, Default, CandidType, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Percentage<T>(T);

pub type PercentagePrecision = u16; // Change this if higher precision is desired

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

/// A helper type implementing Storable for all
/// serde-serializable types using the CBOR encoding.
#[derive(Default)]
pub struct Cbor<T>(pub T)
where
    T: serde::Serialize + serde::de::DeserializeOwned;

impl<T> std::ops::Deref for Cbor<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Storable for Cbor<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![];
        #[allow(clippy::unwrap_used)]
        ciborium::ser::into_writer(&self.0, &mut buf).unwrap();
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        #[allow(clippy::unwrap_used)]
        Self(ciborium::de::from_reader(bytes.as_ref()).unwrap())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StablePrincipal(Blob<29>);

impl From<Principal> for StablePrincipal {
    fn from(caller: Principal) -> Self {
        #[allow(clippy::unwrap_used)]
        Self(Blob::try_from(caller.as_slice()).unwrap())
    }
}

impl From<&Principal> for StablePrincipal {
    fn from(caller: &Principal) -> Self {
        #[allow(clippy::unwrap_used)]
        Self(Blob::try_from(caller.as_slice()).unwrap())
    }
}

impl From<StablePrincipal> for Principal {
    fn from(caller: StablePrincipal) -> Self {
        #[allow(clippy::unwrap_used)]
        Principal::try_from(caller.0.as_slice()).unwrap()
    }
}

impl From<&StablePrincipal> for Principal {
    fn from(caller: &StablePrincipal) -> Self {
        #[allow(clippy::unwrap_used)]
        Principal::try_from(caller.0.as_slice()).unwrap()
    }
}

impl Storable for StablePrincipal {
    const BOUND: Bound = Bound::Bounded {
        max_size: 29,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Borrowed(self.0.as_slice())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        #[allow(clippy::unwrap_used)]
        Self(Blob::try_from(bytes.as_ref()).unwrap())
    }
}
