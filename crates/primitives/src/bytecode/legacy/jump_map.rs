use crate::{hex, keccak256, Bytes, B256, KECCAK_EMPTY};
use alloc::{sync::Arc, vec::Vec};
use bitvec::{
    prelude::{bitvec, Lsb0},
    vec::BitVec,
};
use core::fmt::Debug;

/// A map of valid `jump` destinations.
#[derive(Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JumpMap(pub Arc<BitVec<u8>>);

impl Debug for JumpMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("JumpMap")
            .field("map", &hex::encode(self.0.as_raw_slice()))
            .finish()
    }
}

impl JumpMap {
    /// Get the raw bytes of the jump map
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_raw_slice()
    }

    /// Construct a jumpa map from raw bytes
    #[inline]
    pub fn from_slice(slice: &[u8]) -> Self {
        Self(Arc::new(BitVec::from_slice(slice)))
    }

    /// Check if `pc` is a valid jump destination.
    #[inline]
    pub fn is_valid(&self, pc: usize) -> bool {
        pc < self.0.len() && self.0[pc]
    }
}
