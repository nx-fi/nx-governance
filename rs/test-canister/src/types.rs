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

/// Config of the canister.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct Config {
    /// Name of the canister.
    pub name: String,
    /// Description of the canister.
    pub description: String,
    /// Whether the canister has been initialized.
    pub initialized: bool,
    /// Counter for testing
    pub counter: u64,
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
