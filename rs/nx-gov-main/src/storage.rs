use crate::execution::*;
use crate::proposal::*;
use crate::types::*;

use candid::{Decode, Encode, Principal};
use ic_stable_structures::{
    storable::{Blob, Bound},
    Storable,
};
use std::borrow::Cow;

impl Storable for Proposal {
    const BOUND: Bound = Bound::Bounded {
        max_size: 379,
        is_fixed_size: false,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for ProposalPassingThreshold {
    const BOUND: Bound = Bound::Bounded {
        max_size: 25,
        is_fixed_size: true,
    };

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for ProposalMetadata {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for CanisterMessage {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for ProposalPayload {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for ExecResult {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for ProposalExec {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for ProposalRevoke {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
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
        #[allow(clippy::unwrap_used)] // unwrap expected
        Self(Blob::try_from(bytes.as_ref()).unwrap())
    }
}
